//! Logic for the `nbsp_config` table

use axum_extra::extract::cookie::Key;
use base64ct::{Base64, Encoding};
use jsonwebtoken::DecodingKey;
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

    /// Any additional HTML to add at the end of `<head>` in templates
    pub nbsp_html_head_extra: String,

    /// Any additional HTML to add at the end of `<body>` in templates
    pub nbsp_html_body_extra: String,

    /// The key to encrypt private cookies with
    pub nbsp_cookies_key: Key,

    /// The JWT signing key for encoding and decoding JWTs with, encoded with base64
    pub nbsp_jwt_signing_key: String,

    /// The content security policy header to set on every response
    pub nbsp_content_security_policy: String,
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
            nbsp_html_head_extra: String::new(),
            nbsp_html_body_extra: String::new(),
            nbsp_cookies_key: Key::generate(),
            nbsp_jwt_signing_key: String::new(),
            nbsp_content_security_policy: String::new(),
        };

        let mut save_generated_cookies_key = true;

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

            if key == "nbsp_html_head_extra"
                && let Some(value) = value
            {
                config.nbsp_html_head_extra = value.clone();
            }

            if key == "nbsp_html_body_extra"
                && let Some(value) = value
            {
                config.nbsp_html_body_extra = value.clone();
            }

            if key == "nbsp_cookies_key"
                && let Some(value) = value
            {
                config.nbsp_cookies_key =
                    Key::from(Base64::decode_in_place(&mut value.clone().into_bytes()).unwrap());
                save_generated_cookies_key = false;
            }

            if key == "nbsp_jwt_signing_key"
                && let Some(value) = value
            {
                config.nbsp_jwt_signing_key = value.clone();
            }

            if key == "nbsp_content_security_policy"
                && let Some(value) = value
            {
                config.nbsp_content_security_policy = value.clone();
            }
        }

        assert!(
            !config.nbsp_base_url.is_empty(),
            "nbsp_base_url from nbsp_config is empty"
        );

        if save_generated_cookies_key {
            Self::update_value(
                "nbsp_cookies_key",
                &Base64::encode_string(config.nbsp_cookies_key.master()),
                pool,
            )
            .await?;
        }

        if config.nbsp_jwt_signing_key.is_empty() {
            let secret = Key::generate();
            config.nbsp_jwt_signing_key = Base64::encode_string(
                DecodingKey::from_secret(secret.master())
                    .try_get_as_bytes()
                    .unwrap(),
            );
            Self::update_value("nbsp_jwt_signing_key", &config.nbsp_jwt_signing_key, pool).await?;
        }

        Ok(config)
    }

    /// Update a config value with a given key
    ///
    /// This will return an error when the key for the value doesn't exist
    pub async fn update_value(key: &str, value: &str, pool: &PgPool) -> sqlx::Result<()> {
        let query = "UPDATE nbsp_config SET value = $1 WHERE key = $2 RETURNING config_id;";
        let config_id = sqlx::query_scalar::<_, i64>(query)
            .bind(value)
            .bind(key)
            .fetch_one(pool)
            .await?;

        assert!(
            config_id > 0,
            "expected update_value config_id to be > 0, key={key}"
        );

        Ok(())
    }
}
