//! All [`sqlx`] database code

use std::time::Duration;

use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

mod config;

pub use config::NbspConfig;

/// Create the [`sqlx::PgPool`] and run the database migrations.
pub async fn initialize() -> PgPool {
    let mut conn_opts = PgConnectOptions::new();

    #[cfg(debug_assertions)]
    {
        // In development set the default conn_opts to use the dev database
        // They can still be overridden using the environment variables should it be needed
        conn_opts = conn_opts
            .database("nbsp")
            .username("nbsp")
            .password("nbsp")
            .host(&format!("{}/podman/postgresql", env!("CARGO_MANIFEST_DIR")));
    }

    if let Ok(connection_string) = std::env::var("NBSP_PG_CONNECTION_STRING") {
        conn_opts = connection_string
            .parse()
            .expect("NBSP_PG_CONNECTION_STRING must be a valid connection string");
    }

    if let Ok(database) = std::env::var("NBSP_PG_DATABASE") {
        conn_opts = conn_opts.database(&database);
    }

    if let Ok(username) = std::env::var("NBSP_PG_USERNAME") {
        conn_opts = conn_opts.username(&username);
    }

    if let Ok(password) = std::env::var("NBSP_PG_PASSWORD") {
        conn_opts = conn_opts.password(&password);
    }

    if let Ok(host) = std::env::var("NBSP_PG_HOST") {
        conn_opts = conn_opts.host(&host);
    }

    if let Ok(port) = std::env::var("NBSP_PG_PORT") {
        conn_opts = conn_opts.port(port.parse().expect("NBSP_PG_PORT must be a u16"));
    }

    let pool_opts = PgPoolOptions::new().acquire_timeout(Duration::from_secs(5));

    let pool = pool_opts
        .connect_with(conn_opts)
        .await
        .expect("failed to connect to PostgreSQL");

    sqlx::migrate!("./migrations/")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    pool
}
