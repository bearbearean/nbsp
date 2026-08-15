//! All [`askama`] HTML template definitions

use askama::Template;

/// The homepage template
#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct Homepage {
    /// An optional HTML section to show as a yellow notice on the homepage
    pub nbsp_homepage_notice: Option<String>,
}
