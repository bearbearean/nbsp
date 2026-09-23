//! All axum routes under `/post/...`

use axum::{
    Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Redirect,
};
use serde::Deserialize;

use crate::{
    GlobalState,
    database::{Post, User},
    jwt::auth::MustAuth,
    prelude::*,
    templates::{HttpStatusPage, PostNew, PostView},
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
    let creator_user_id = auth.user.user_id;

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
        creator_user_id,
        &form.title,
        &form.markdown,
        &html,
        &gs.pool,
    )
    .await?;

    let post_url = format!("/post/view/{}", post.post_id);
    Ok(Redirect::to(&post_url).into_response())
}

/// The route for `GET /post/view/{post_id}`
pub async fn post_view(
    State(gs): State<GlobalState>,
    auth: MustAuth,
    headers: HeaderMap,
    Path(post_id): Path<i64>,
) -> WebResult {
    match Post::optional_find_by_post_id(post_id, &gs.pool).await? {
        Some(post) => {
            let post_creator_username = User::get_username(post.creator_user_id, &gs.pool).await?;
            html(PostView {
                auth: auth.into_auth(),
                config: gs.config,
                post,
                post_creator_username,
            })
        }
        None => html_with_status(
            HttpStatusPage {
                config: gs.config,
                title: "Post not found - HTTP 404",
                description: "There doesn't seem to be a post by that ID.",
                x_request_id: headers
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or(""),
            },
            StatusCode::NOT_FOUND,
        ),
    }
}
