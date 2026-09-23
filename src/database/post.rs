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
        creator_user_id: i64,
        title: &str,
        markdown: &str,
        html: &str,
        pool: &PgPool,
    ) -> sqlx::Result<Self> {
        let query = r#"
INSERT INTO posts (creator_user_id, title, markdown_source, html_rendered)
VALUES ($1, $2, $3, $4)
RETURNING *;
"#;
        sqlx::query_as(query)
            .bind(creator_user_id)
            .bind(title)
            .bind(markdown)
            .bind(html)
            .fetch_one(pool)
            .await
    }

    /// Find a post by its `post_id`, returning `None` if it cannot be found
    pub async fn optional_find_by_post_id(
        post_id: i64,
        pool: &PgPool,
    ) -> sqlx::Result<Option<Self>> {
        let query = "SELECT * FROM posts WHERE post_id = $1;";
        sqlx::query_as(query)
            .bind(post_id)
            .fetch_optional(pool)
            .await
    }
}

/// A minimal representation of [`Post`] that only contains the necessary information to show posts
/// in a list
#[derive(FromRow)]
pub struct PostListItem {
    /// The ID of the post (primary key)
    pub post_id: i64,
    /// The username of the creator of the post
    pub creator_username: String,
    /// The timestamp when the post was created
    pub created_at: DateTime<Utc>,
    /// The plaintext title of the post
    pub title: String,
}

impl PostListItem {
    /// Get the 25 most recently posted posts
    pub async fn get_recent_25(pool: &PgPool) -> sqlx::Result<Vec<Self>> {
        let query = r#"
SELECT
    p.post_id,
    p.created_at,
    p.title,
    u.username creator_username
FROM posts p
JOIN users u ON p.creator_user_id = u.user_id
ORDER BY p.created_at DESC LIMIT 25;
"#;
        sqlx::query_as(query).fetch_all(pool).await
    }
}
