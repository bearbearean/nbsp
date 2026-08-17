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
