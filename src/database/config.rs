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

    /// The title/name of the instance.
    pub nbsp_community_title: String,

    /// The subtitle/byline of the instance.
    pub nbsp_community_subtitle: String,
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
            nbsp_community_title: String::new(),
            nbsp_community_subtitle: String::new(),
        };

        for (key, value) in &rows {
            if key == "nbsp_base_url"
                && let Some(value) = value
            {
                config.nbsp_base_url = value.clone();
            }

            if key == "nbsp_homepage_notice" {
                config.nbsp_homepage_notice = value.clone();
            }

            if key == "nbsp_community_title" {
                config.nbsp_community_title = value
                    .clone()
                    .unwrap_or_else(|| "non-breaking space".to_string());
            }

            if key == "nbsp_community_subtitle" {
                config.nbsp_community_subtitle = value
                    .clone()
                    .unwrap_or_else(|| "a thoughtful community forum platform".to_string());
            }
        }

        assert!(
            !config.nbsp_base_url.is_empty(),
            "nbsp_base_url from nbsp_config is empty"
        );

        Ok(config)
    }
}
