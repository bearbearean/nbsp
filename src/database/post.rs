//! Logic for the `posts` table

use std::ops::RangeInclusive;

use sqlx::prelude::FromRow;

use crate::prelude::*;

/// A post on nbsp
#[derive(FromRow)]
pub struct Post {
    /// The ID of the post (primary key)
    pub post_id: i64,
    /// The ID of the user that created the post (foreign key)
    pub creator_user_id: i64,
    /// The timestamp when the post was created
    pub created_at: DateTime<Utc>,
    /// The plaintext title of the post
    pub title: String,
    /// The Markdown source of the post body
    pub markdown_source: String,
    /// The rendered HTML of the post body
    pub html_rendered: String,
}

impl Post {
    /// The allowed range of title length: at least 1 and at most 200 characters
    pub const TITLE_LENGTH: RangeInclusive<usize> = 1..=200;
    /// The allowed range of Markdown length: at least 1 and at most 10_000 characters
    pub const MARKDOWN_LENGTH: RangeInclusive<usize> = 1..=10_000;

    /// Validate a given title matches the expected format
    pub fn validate_title(title: &str) -> bool {
        let len = title.len();

        Self::TITLE_LENGTH.contains(&len)
    }

    /// Validate a given title matches the expected format
    pub fn validate_markdown(markdown: &str) -> bool {
        let len = markdown.len();

        Self::MARKDOWN_LENGTH.contains(&len)
    }

    /// Create a new post and save it in the database
    pub async fn create_new(
        user_creator_id: i64,
        title: &str,
        markdown: &str,
        html: &str,
        pool: &PgPool,
    ) -> sqlx::Result<Self> {
        let query = r#"
INSERT INTO posts (user_creator_id, title, markdown_source, html_rendered)
VALUES ($1, $2, $3, $4)
RETURNING *;
"#;
        sqlx::query_as(query)
            .bind(user_creator_id)
            .bind(title)
            .bind(markdown)
            .bind(html)
            .fetch_one(pool)
            .await
    }
}
