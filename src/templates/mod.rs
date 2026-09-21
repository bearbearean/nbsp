//! All [`askama`] HTML template definitions

use askama::Template;

use crate::{
    database::{Invite, NbspConfig, Post, PostListItem, User, UserInviteSettings},
    jwt::auth::Auth,
    utilities::{LoginUserError, PostNewError, RegisterUserError},
};

/// The homepage template
#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct Homepage {
    /// The [`NbspConfig`] for the instance
    pub config: NbspConfig,
    /// The authentication context
    pub auth: Auth,
    /// Recent posts to show to a logged in user
    pub posts: Option<Vec<PostListItem>>,
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
    /// An optional error message to show as feedback for the user
    pub form_error_message: Option<RegisterUserError>,
}

/// The account login template
#[derive(Template)]
#[template(path = "pages/login.html")]
pub struct AccountLogin {
    /// The [`NbspConfig`] for the instance
    pub config: NbspConfig,
    /// The URL to redirect back to after login
    pub redirect: Option<String>,
    /// An optional error message to show as feedback for the user
    pub form_error_message: Option<LoginUserError>,
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

/// The user profile template
#[derive(Template)]
#[template(path = "pages/user_invite_settings.html")]
pub struct AccountInvites {
    /// The [`NbspConfig`] for the instance
    pub config: NbspConfig,
    /// The authentication context, this contains the authenticated user
    pub auth: Auth,
    /// The user's invite settings
    pub settings: UserInviteSettings,
    /// The existing and not yet consumed invite codes created by the user
    pub invites: Vec<Invite>,
}

/// The user profile template
#[derive(Template)]
#[template(path = "pages/post_new.html")]
pub struct PostNew {
    /// The [`NbspConfig`] for the instance
    pub config: NbspConfig,
    /// The authentication context, this contains the authenticated user
    pub auth: Auth,
    /// An optional error message to show as feedback for the user
    pub form_error_message: Option<PostNewError>,
    /// A title to prefill in the title input
    pub prefilled_title: Option<String>,
    /// A Markdown body to prefill in the body textarea
    pub prefilled_markdown: Option<String>,
}

/// The post view template
#[derive(Template)]
#[template(path = "pages/post_view.html")]
pub struct PostView {
    /// The [`NbspConfig`] for the instance
    pub config: NbspConfig,
    /// The authentication context, this contains the authenticated user
    pub auth: Auth,
    /// The post to view
    pub post: Post,
    /// The username of the creator of the post
    pub post_creator_username: String,
}
