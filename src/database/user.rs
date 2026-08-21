//! Logic for the `users` table

use std::{ops::RangeInclusive, sync::LazyLock};

use chrono::{DateTime, Utc};
use regex::Regex;
use sqlx::{PgPool, prelude::FromRow, types::Uuid};

use crate::database::{Invite, RefreshToken};

/// A nbsp user account
#[derive(Clone, FromRow)]
pub struct User {
    /// The ID of the user (primary key)
    pub user_id: i64,
    /// The name of the user (unique)
    pub username: String,
    /// The timestamp of when the user account was created
    pub created_at: DateTime<Utc>,
    /// The hashed password of the user
    ///
    /// If this is `None` then the user must not be allowed to be logged in as
    pub password_hash: Option<String>,
}

impl User {
    /// The allowed range of username length: at least 3 and at most 40 characters
    pub const USERNAME_LENGTH: RangeInclusive<usize> = 3..=40;
    /// The allowed username regular expression
    #[allow(clippy::declare_interior_mutable_const)]
    pub const USERNAME_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("^[a-zA-Z0-9][a-zA-Z0-9_]+[a-zA-Z0-9]$").unwrap());
    /// The allowed range of password length: at least 10 and at most 200 characters
    pub const PASSWORD_LENGTH: RangeInclusive<usize> = 10..=200;
    /// A list of regular expressions that specifies the allowed characters in a password
    #[allow(clippy::declare_interior_mutable_const)]
    pub const PASSWORD_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
        vec![
            Regex::new("[a-z]").unwrap(),        // Lowercase letter
            Regex::new("[A-Z]").unwrap(),        // Uppercase letter
            Regex::new("[0-9]").unwrap(),        // Numbers
            Regex::new("[^a-zA-Z0-9]").unwrap(), // Any other character apart from the previous sets
        ]
    });

    /// Validate a given username matches the expected format
    pub fn validate_username(username: &str) -> bool {
        let len = username.len();

        #[allow(clippy::borrow_interior_mutable_const)]
        {
            Self::USERNAME_LENGTH.contains(&len) && Self::USERNAME_REGEX.is_match(username)
        }
    }

    /// Validate a given password matches the expected format
    pub fn validate_password(password: &str) -> bool {
        let len = password.len();

        #[allow(clippy::borrow_interior_mutable_const)]
        {
            Self::PASSWORD_LENGTH.contains(&len)
                && Self::PASSWORD_REGEXES
                    .iter()
                    .all(|re| re.is_match(password))
        }
    }

    /// Check if a username is available for registration
    pub async fn is_username_available(username: &str, pool: &PgPool) -> sqlx::Result<bool> {
        let query = "SELECT user_id FROM users WHERE lower(username) = lower($1);";
        let existing_user_id = sqlx::query_scalar::<_, i64>(query)
            .bind(username)
            .fetch_optional(pool)
            .await?;
        Ok(existing_user_id.is_none())
    }

    /// Create a new user and consume the invite code. This function does not perform input
    /// validation, that is expected to be done **before** this function is called
    ///
    /// All database operations happen in a transaction, if this function fails for any reason no
    /// changes will have been made in the database
    pub async fn register_account(
        username: &str,
        password_hash: &str,
        invite: &Uuid,
        pool: &PgPool,
    ) -> sqlx::Result<(Self, RefreshToken)> {
        let mut txn = pool.begin().await?;
        let query = "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING *;";
        let user = sqlx::query_as::<_, Self>(query)
            .bind(username)
            .bind(password_hash)
            .fetch_one(&mut *txn)
            .await?;

        let invite = Invite::consume_code(user.user_id, invite, &mut txn).await?;
        assert_eq!(invite.user_consumer_id, Some(user.user_id));
        assert!(invite.consumed_at.is_some());

        let refresh_token = RefreshToken::new_for_user(user.user_id, &mut txn).await?;

        txn.commit().await?;
        Ok((user, refresh_token))
    }

    /// Find a user by their `user_id`, returning an `Err` if they cannot be found
    pub async fn find_by_id(user_id: i64, pool: &PgPool) -> sqlx::Result<Self> {
        let query = "SELECT * FROM users WHERE user_id = $1;";
        sqlx::query_as(query).bind(user_id).fetch_one(pool).await
    }

    /// Find a user by their `user_id`, returning `None` if they cannot be found
    pub async fn optional_find_by_id(user_id: i64, pool: &PgPool) -> sqlx::Result<Option<Self>> {
        let query = "SELECT * FROM users WHERE user_id = $1;";
        sqlx::query_as(query)
            .bind(user_id)
            .fetch_optional(pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_validate_username() {
        let username = "B";
        assert!(
            !User::validate_username(username),
            "username too short: {username}"
        );

        let username = format!("1{}", "A".repeat(*User::USERNAME_LENGTH.end()));
        assert!(
            !User::validate_username(&username),
            "username too long: {username}"
        );

        let username = "A".repeat(*User::USERNAME_LENGTH.start());
        assert!(
            User::validate_username(&username),
            "username ok min length: {username}"
        );

        let username = "A".repeat(*User::USERNAME_LENGTH.end());
        assert!(
            User::validate_username(&username),
            "username ok max length: {username}"
        );

        let username = "_nbsp";
        assert!(
            !User::validate_username(username),
            "username starts with underscore: {username}"
        );

        let username = "nbsp_";
        assert!(
            !User::validate_username(username),
            "username ends with underscore: {username}"
        );

        let username = "n_b_s_p";
        assert!(
            User::validate_username(username),
            "username ok with underscores: {username}"
        );
    }

    #[test]
    fn test_user_validate_password() {
        let password = "B";
        assert!(
            !User::validate_password(password),
            "password too short: {password}"
        );

        let password = format!("1_a{}", "A".repeat(*User::PASSWORD_LENGTH.end()));
        assert!(
            !User::validate_password(&password),
            "password too long: {password}"
        );

        let password = "1234567_aA";
        assert!(
            User::validate_password(password),
            "password ok min length: {password}"
        );

        let password = "1234567_aA".repeat(20);
        assert!(
            User::validate_password(&password),
            "password ok max length: {password}"
        );

        let password = "123456789_A";
        assert!(
            !User::validate_password(password),
            "password missing lowercase: {password}"
        );

        let password = "123456789a_";
        assert!(
            !User::validate_password(password),
            "password missing uppercase: {password}"
        );

        let password = "abcdefghi_aA";
        assert!(
            !User::validate_password(password),
            "password missing number: {password}"
        );

        let password = "123456789aA";
        assert!(
            !User::validate_password(password),
            "password missing other character: {password}"
        );
    }
}
