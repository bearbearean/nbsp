//! All [`askama`] HTML template definitions

use askama::Template;

/// The homepage template
#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct Homepage {}
