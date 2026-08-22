//! All [`askama`] HTML template definitions

use askama::Template;

use crate::{
    auth::Auth,
    database::{NbspConfig, User},
};

/// The homepage template
#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct Homepage {
    /// The [`NbspConfig`] for the instance
    pub config: NbspConfig,
    /// The authentication context
    pub auth: Auth,
}

/// A generic template for HTTP status codes
#[derive(Template)]
#[template(path = "pages/http_status.html")]
pub struct HttpStatusPage<'a> {
    /// The [`NbspConfig`] for the instance
    pub config: NbspConfig,
    /// The HTTP status code title
    pub title: &'a str,
    /// A short description of what the problem is
    pub description: &'a str,
    /// The x-request-id HTTP header, in case further investigation is needed
    pub x_request_id: &'a str,
}

/// The account registration template
#[derive(Template)]
#[template(path = "pages/register.html")]
pub struct AccountRegister {
    /// The [`NbspConfig`] for the instance
    pub config: NbspConfig,
    /// An invite code to prefill in the invite input
    pub prefilled_invite_code: Option<String>,
}

/// The account login template
#[derive(Template)]
#[template(path = "pages/login.html")]
pub struct AccountLogin {
    /// The [`NbspConfig`] for the instance
    pub config: NbspConfig,
    /// The URL to redirect back to after login
    pub redirect: Option<String>,
}

/// The user profile template
#[derive(Template)]
#[template(path = "pages/user.html")]
pub struct UserProfile {
    /// The [`NbspConfig`] for the instance
    pub config: NbspConfig,
    /// The authentication context, this contains the authenticated user
    pub auth: Auth,
    /// The user to view the profile of
    pub target_user: User,
}
