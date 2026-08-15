//! Logic for the `nbsp_config` table

use sqlx::PgPool;

/// A select set of data from the `nbsp_config`, to be used in axum's global state
#[derive(Clone)]
pub struct NbspConfig {
    /// The canonical base URL for the running nbsp instance
    ///
    /// This should start with `https://` and be the "main" URL of the website
    ///
    /// Defaults to `https://nbsp.example.com` in the `20260816_000_initialize.up.sql` file
    pub nbsp_base_url: String,

    /// A notice to put at the top of the homepage in a yellow box
    ///
    /// If set to `NULL`/`None` no notice will be shown
    pub nbsp_homepage_notice: Option<String>,
}

impl NbspConfig {
    /// Load the [`NbspConfig`]
    pub async fn load(pool: &PgPool) -> sqlx::Result<Self> {
        let query = "SELECT key, value FROM nbsp_config;";
        let rows = sqlx::query_as::<_, (String, Option<String>)>(query)
            .fetch_all(pool)
            .await?;

        let mut config = Self {
            nbsp_base_url: String::new(),
            nbsp_homepage_notice: None,
        };

        for (key, value) in rows {
            if key == "nbsp_base_url"
                && let Some(value) = value.as_ref()
            {
                config.nbsp_base_url = value.to_string();
            }

            if key == "nbsp_homepage_notice" {
                config.nbsp_homepage_notice = value;
            }
        }

        assert!(
            !config.nbsp_base_url.is_empty(),
            "nbsp_base_url from nbsp_config is empty"
        );

        Ok(config)
    }
}
