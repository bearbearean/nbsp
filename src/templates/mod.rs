//! All [`askama`] HTML template definitions

use askama::Template;

use crate::database::NbspConfig;

/// The homepage template
#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct Homepage {
    /// The [`NbspConfig`] for the instance
    pub config: NbspConfig,
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
