//! Logic for the `invites` table

use chrono::{DateTime, Utc};
use sqlx::{PgPool, PgTransaction, prelude::FromRow, types::Uuid};

/// An invite to create a nbsp account with
#[derive(FromRow)]
pub struct Invite {
    /// The ID of the invite (primary key)
    pub invite_id: i64,
    /// The actual invite code to use during account registration
    pub invite_code: Uuid,
    /// The ID of creator of the invite code, this is the person that generated the code
    pub user_creator_id: i64,
    /// The timestamp of when the invite was created
    pub created_at: DateTime<Utc>,
    /// The ID of the consumer of the invite code, this is the person that used the code to create
    /// their account
    ///
    /// When this is `Some()` the invite code must not be able to be used again
    pub user_consumer_id: Option<i64>,
    /// The timestamp of when the invite was consumed
    ///
    /// When this is `Some()` the invite code must not be able to be used again
    pub consumed_at: Option<DateTime<Utc>>,
}

impl Invite {
    /// Validate a given invite code matches the expected format
    pub fn validate_invite(invite: &str) -> bool {
        Uuid::try_parse(invite).is_ok()
    }

    /// Check that a given invite code exists in the database and hasn't been consumed
    ///
    /// An invite code is considered not consumed when both `user_consumer_id` AND `consumed_at` are
    /// `NULL`. If either or both are not null then the code has been used
    pub async fn is_invite_available(invite: &Uuid, pool: &PgPool) -> sqlx::Result<bool> {
        let query = r#"
SELECT invite_id FROM invites
WHERE
    invite_code = $1 AND
    user_consumer_id IS NULL AND
    consumed_at IS NULL;
"#;
        let invite_id = sqlx::query_scalar::<_, i64>(query)
            .bind(invite)
            .fetch_optional(pool)
            .await?;
        Ok(invite_id.is_some())
    }

    /// Consume an invite code
    ///
    /// This takes a [`PgTransaction`] rather than a [`PgPool`] because this function should only
    /// ever be used when registering a new user account
    pub async fn consume_code(
        user_consumer_id: i64,
        invite: &Uuid,
        txn: &mut PgTransaction<'_>,
    ) -> sqlx::Result<Self> {
        let query = r#"
UPDATE invites
SET
    user_consumer_id = $1,
    consumed_at = now()
WHERE
    invite_code = $2 AND
    user_consumer_id IS NULL AND
    consumed_at IS NULL
RETURNING *;
"#;
        let invite = sqlx::query_as(query)
            .bind(user_consumer_id)
            .bind(invite)
            .fetch_one(txn.as_mut())
            .await?;

        Ok(invite)
    }
}
