//! Database logic for JWT refresh tokens

use sqlx::{PgTransaction, prelude::FromRow, types::Uuid};

use crate::prelude::*;

/// A refresh token to automatically refresh JWT authentication with
#[derive(FromRow)]
pub struct RefreshToken {
    /// The ID of the refresh token (primary key)
    pub token_id: i64,
    /// The ID of the [`crate::database::User`] the refresh token belongs to
    pub user_id: i64,
    /// The actual contents of the refresh token that will be in the refresh cookie
    pub refresh_token: Uuid,
    /// When the refresh token was created
    pub created_at: DateTime<Utc>,
    /// When the refresh token was last used
    pub last_used_at: DateTime<Utc>,
}

impl RefreshToken {
    /// Create a new refresh token for a given user
    ///
    /// This takes a [`PgTransaction`] instead of the usual [`PgPool`] since it is likely that
    /// other database operations need to happen alongside creating a refresh token
    pub async fn new_for_user(user_id: i64, txn: &mut PgTransaction<'_>) -> sqlx::Result<Self> {
        let query = "INSERT INTO refresh_tokens (user_id) VALUES ($1) RETURNING *;";
        sqlx::query_as(query)
            .bind(user_id)
            .fetch_one(txn.as_mut())
            .await
    }

    /// Find a refresh token by its token value, returning `None` if it cannot be found
    pub async fn optional_find_by_token(token: &Uuid, pool: &PgPool) -> sqlx::Result<Option<Self>> {
        let query = "SELECT * FROM refresh_tokens WHERE refresh_token = $1;";
        sqlx::query_as(query).bind(token).fetch_optional(pool).await
    }

    /// Update the timestamp when a refresh token was last used to now
    pub async fn update_last_used(self, pool: &PgPool) -> sqlx::Result<Self> {
        let query =
            "UPDATE refresh_tokens SET last_used_at = now() WHERE token_id = $1 RETURNING *;";
        sqlx::query_as(query)
            .bind(self.token_id)
            .fetch_one(pool)
            .await
    }

    /// Delete a refresh token from the database, this does not error if nothing is deleted. A
    /// warning will be logged however
    pub async fn optional_delete(token: &Uuid, pool: &PgPool) -> sqlx::Result<()> {
        let query = "DELETE FROM refresh_tokens WHERE refresh_token = $1 RETURNING token_id;";
        let token_id = sqlx::query_scalar::<_, i64>(query)
            .bind(token)
            .fetch_optional(pool)
            .await?;

        if token_id.is_none() {
            tracing::warn!(
                token = ?token,
                "tried to delete refresh token that does not exist in the database"
            );
        }

        Ok(())
    }
}
