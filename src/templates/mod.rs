//! All [`askama`] HTML template definitions

use askama::Template;

/// The homepage template
#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct Homepage {
    /// An optional HTML section to show as a yellow notice on the homepage
    pub nbsp_homepage_notice: Option<String>,
    /// The title of the nbsp instance.
    pub nbsp_community_title: String,
    /// The subtitle of the nbsp instance.
    pub nbsp_community_subtitle: String,
}

/// A generic template for HTTP status codes
#[derive(Template)]
#[template(path = "pages/http_status.html")]
pub struct HttpStatusPage<'a> {
    /// The title of the nbsp instance.
    pub nbsp_community_title: String,
    /// The subtitle of the nbsp instance.
    pub nbsp_community_subtitle: String,
    /// The HTTP status code title
    pub title: &'a str,
    /// A short description of what the problem is
    pub description: &'a str,
    /// The x-request-id HTTP header, in case further investigation is needed
    pub x_request_id: &'a str,
}
