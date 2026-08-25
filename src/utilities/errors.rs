//! Custom error enums and helpers

use std::fmt::Display;

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
