//! Logic for the `user_invite_settings` table

use sqlx::{PgPool, PgTransaction, prelude::FromRow};

/// The user's invite settings
#[derive(Clone, FromRow)]
pub struct UserInviteSettings {
    /// The ID of the user invite setting (primary key)
    pub setting_id: i64,
    /// The ID of the user (foreign key)
    pub user_id: i64,
    /// How many invite codes the user has available to create (defaults to 0)
    pub available_invite_count: i64,
}

impl UserInviteSettings {
    /// Get the [`UserInviteSettings`] for a given `user_id`. This will do an `INSERT` if the user
    /// does not yet have a record in this table.
    pub async fn get_by_user_id(user_id: i64, pool: &PgPool) -> sqlx::Result<Self> {
        let query = "SELECT * FROM user_invite_settings WHERE user_id = $1;";
        let user_invite_settings = sqlx::query_as::<_, Self>(query)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

        if let Some(user_invite_settings) = user_invite_settings {
            Ok(user_invite_settings)
        } else {
            Self::create_new(user_id, pool).await
        }
    }

    /// Create a [`UserInviteSettings`] record for a user. Note that this will error if the user
    /// already has a record in this table. Use [`UserInviteSettings::get_by_user_id`] instead to
    /// get an existing or create a new record in one function call.
    pub async fn create_new(user_id: i64, pool: &PgPool) -> sqlx::Result<Self> {
        let query = "INSERT INTO user_invite_settings (user_id) VALUES ($1) RETURNING *;";
        sqlx::query_as::<_, Self>(query)
            .bind(user_id)
            .fetch_one(pool)
            .await
    }

    /// Update the `available_invite_count` of an existing user invite settings record. This will
    /// error if there is no existing record to update.
    pub async fn save_invite_count(
        setting_id: i64,
        available_invite_count: i64,
        txn: &mut PgTransaction<'_>,
    ) -> sqlx::Result<Self> {
        let query = r#"
UPDATE user_invite_settings
SET available_invite_count = $1
WHERE setting_id = $2
RETURNING *;
"#;
        sqlx::query_as(query)
            .bind(available_invite_count)
            .bind(setting_id)
            .fetch_one(txn.as_mut())
            .await
    }
}
