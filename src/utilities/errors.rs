//! Custom error enums and helpers

use std::fmt::Display;

use crate::database::Post;

/// Errors that can happen when a user is registering an account, to be used as a feedback message
pub enum RegisterUserError {
    /// An invalid invite code, can be an unknown code, already consumed, etc.
    InviteCode,
    /// Invalid form data was submitted, such as username that does not match the required format...
    InvalidForm,
    /// The requested username has already been taken
    ExistingUsername,
    /// Unknown error
    Unknown,
}

impl Display for RegisterUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InviteCode => "Your invite code is invalid or has already been used",
            Self::InvalidForm => "Your submitted data does not match the format requirements",
            Self::ExistingUsername => "Your requested username is already in use",
            Self::Unknown => "An unknown error has occurred, please try again",
        };

        writeln!(f, "{message}")
    }
}

/// Errors that can happen when a user is logging into an account, to be used as a feedback message
pub enum LoginUserError {
    /// Combination of username and password do not match
    IncorrectLogin,
}

impl Display for LoginUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::IncorrectLogin => "Incorrect username and password",
        };

        writeln!(f, "{message}")
    }
}

/// Errors that can happen when a user creates a new post
pub enum PostNewError {
    /// Title is too short or too long
    TitleLength,
    /// Markdown is too short or too long
    MarkdownLength,
}

impl Display for PostNewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::TitleLength => &format!(
                "The title must be at least {} and at most {} characters long",
                Post::TITLE_LENGTH.start(),
                Post::TITLE_LENGTH.end(),
            ),
            Self::MarkdownLength => &format!(
                "The Markdown body must be at least {} and at most {} characters long",
                Post::MARKDOWN_LENGTH.start(),
                Post::MARKDOWN_LENGTH.end()
            ),
        };

        writeln!(f, "{message}")
    }
}
