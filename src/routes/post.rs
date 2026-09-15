//! All axum routes under `/post/...`

use axum::{Form, extract::State, response::Redirect};
use serde::Deserialize;

use crate::{
    GlobalState,
    database::Post,
    jwt::auth::MustAuth,
    prelude::*,
    templates::PostNew,
    utilities::{PostNewError, html, html_with_status, render_markdown},
};

/// The route for `GET /post/new`
pub async fn post_new(State(gs): State<GlobalState>, auth: MustAuth) -> WebResult {
    html(PostNew {
        config: gs.config,
        auth: auth.into_auth(),
        form_error_message: None,
        prefilled_title: None,
        prefilled_markdown: None,
    })
}

/// Expected input form for `POST /post/new`
#[derive(Deserialize)]
pub struct PostNewForm {
    /// The post's title
    pub title: String,
    /// The post's Markdown body
    pub markdown: String,
}

/// The route for `POST /post/new`
pub async fn do_post_new(
    State(gs): State<GlobalState>,
    auth: MustAuth,
    Form(form): Form<PostNewForm>,
) -> WebResult {
    let user_creator_id = auth.user.user_id;

    let err_status = StatusCode::UNPROCESSABLE_ENTITY;
    let mut template = PostNew {
        config: gs.config,
        auth: auth.into_auth(),
        form_error_message: None,
        prefilled_title: Some(form.title.clone()),
        prefilled_markdown: Some(form.markdown.clone()),
    };

    if !Post::validate_title(&form.title) {
        template.form_error_message = Some(PostNewError::TitleLength);
        return html_with_status(template, err_status);
    }

    if !Post::validate_markdown(&form.markdown) {
        template.form_error_message = Some(PostNewError::MarkdownLength);
        return html_with_status(template, err_status);
    }

    let html = render_markdown(&form.markdown);
    let post = Post::create_new(
        user_creator_id,
        &form.title,
        &form.markdown,
        &html,
        &gs.pool,
    )
    .await?;

    let post_url = format!("/post/view?id={}", post.post_id);
    Ok(Redirect::to(&post_url).into_response())
}
