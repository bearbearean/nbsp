//! All axum routes under `/account/...`

use axum::{
    Form,
    extract::{Query, State},
    response::Redirect,
};
use axum_extra::extract::PrivateCookieJar;
use serde::Deserialize;
use sqlx::types::Uuid;

use crate::{
    GlobalState,
    database::{Invite, RefreshToken, User, UserInviteSettings},
    jwt::{
        auth::{MustAuth, MustNotBeAuthed},
        cookies::{COOKIE_REFRESH, COOKIE_REFRESH_MAX_AGE, build_cookie, clear_cookie_jar},
        generate_jwt_cookie,
    },
    prelude::*,
    templates::{AccountInvites, AccountLogin, AccountRegister},
    utilities::{
        LoginUserError, RegisterUserError, hash_password, html, html_with_status, verify_password,
    },
};

/// Query parameters for `GET /account/register`

#[derive(Deserialize)]
pub struct AccountRegisterQueryParams {
    /// An optional invite code to prefill in the account registration form
    pub invite: Option<String>,
}

/// The GET handler for `/account/register`
pub async fn account_register(
    params: Query<AccountRegisterQueryParams>,
    _auth: MustNotBeAuthed,
    State(gs): State<GlobalState>,
) -> WebResult {
    html(AccountRegister {
        config: gs.config,
        prefilled_invite_code: params.0.invite,
        form_error_message: None,
    })
}

/// Expected input form for `POST /account/register`
#[derive(Deserialize)]
pub struct AccountRegisterForm {
    /// Value of the username form input
    pub username: String,
    /// Value of the password form input
    pub password: String,
    /// Value of the confirm_password form input
    pub confirm_password: String,
    /// Value of the invite form input
    pub invite: String,
}

impl AccountRegisterForm {
    /// Validate an account registration form input matches all the expected formats
    pub fn validate(&self) -> bool {
        self.password == self.confirm_password
            && User::validate_username(&self.username)
            && User::validate_password(&self.password)
            && Invite::validate_invite(&self.invite)
    }
}

/// The POST handler for `/account/register`
pub async fn do_account_register(
    State(gs): State<GlobalState>,
    jar: PrivateCookieJar,
    _auth: MustNotBeAuthed,
    Form(form): Form<AccountRegisterForm>,
) -> WebResult {
    let err_status = StatusCode::UNPROCESSABLE_ENTITY;
    let mut template = AccountRegister {
        config: gs.config,
        prefilled_invite_code: Some(form.invite.clone()),
        form_error_message: None,
    };

    let invite = match Uuid::try_parse(&form.invite) {
        Ok(invite) => invite,
        Err(_) => {
            template.form_error_message = Some(RegisterUserError::InviteCode);
            return html_with_status(template, err_status);
        }
    };

    let form_is_valid = form.validate();
    if !form_is_valid {
        tracing::info!("attempted user registration with invalid form details");
        template.form_error_message = Some(RegisterUserError::InvalidForm);
        return html_with_status(template, err_status);
    }

    let username_is_available = User::is_username_available(&form.username, &gs.pool).await?;
    if !username_is_available {
        tracing::info!("attempted user registration with existing username");
        template.form_error_message = Some(RegisterUserError::ExistingUsername);
        return html_with_status(template, err_status);
    }

    let invite_is_available = Invite::is_invite_available(&invite, &gs.pool).await?;
    if !invite_is_available {
        tracing::info!("attempted user registration with unavailable invite code");
        template.form_error_message = Some(RegisterUserError::InviteCode);
        return html_with_status(template, err_status);
    }

    if form_is_valid && username_is_available && invite_is_available {
        let password_hash = hash_password(form.password.as_bytes())?;
        let (user, refresh_token) =
            User::register_account(&form.username, &password_hash, &invite, &gs.pool).await?;
        let jwt_cookie = generate_jwt_cookie(&gs.jwt_encoding_key, user.user_id)?;
        let refresh_cookie = build_cookie(
            COOKIE_REFRESH,
            refresh_token.refresh_token.to_string(),
            Some(COOKIE_REFRESH_MAX_AGE),
        );

        Ok((
            jar.add(jwt_cookie).add(refresh_cookie),
            Redirect::to(&format!("/user/{}", user.username)),
        )
            .into_response())
    } else {
        template.form_error_message = Some(RegisterUserError::Unknown);
        html_with_status(template, err_status)
    }
}

/// The route for `GET /account/invites`
pub async fn account_invites(State(gs): State<GlobalState>, auth: MustAuth) -> WebResult {
    let user_id = auth.user.user_id;
    html(AccountInvites {
        auth: auth.into_auth(),
        config: gs.config,
        settings: UserInviteSettings::get_by_user_id(user_id, &gs.pool).await?,
        invites: Invite::get_available_by_creator(user_id, &gs.pool).await?,
    })
}

/// Query parameters for `/account/login`
#[derive(Deserialize)]
pub struct AccountLoginQueryParams {
    /// An optional URL to redirect back to after the login is done
    pub redirect: Option<String>,
}

/// The GET handler for `/account/login`
pub async fn account_login(
    State(gs): State<GlobalState>,
    Query(params): Query<AccountLoginQueryParams>,
    _auth: MustNotBeAuthed,
) -> WebResult {
    html(AccountLogin {
        config: gs.config,
        redirect: params.redirect,
        form_error_message: None,
    })
}

/// Expected input form for `POST /account/login`
#[derive(Deserialize)]
pub struct AccountLoginForm {
    /// Value of the username form input
    pub username: String,
    /// Value of the password form input
    pub password: String,
}

/// The POST handler for `/account/login`
pub async fn do_account_login(
    State(gs): State<GlobalState>,
    Query(params): Query<AccountLoginQueryParams>,
    jar: PrivateCookieJar,
    _auth: MustNotBeAuthed,
    Form(form): Form<AccountLoginForm>,
) -> WebResult {
    let mut txn = gs.pool.begin().await?;
    let status = StatusCode::UNAUTHORIZED;
    let mut err_template = AccountLogin {
        config: gs.config,
        redirect: params.redirect.clone(),
        form_error_message: None,
    };

    let user = match User::optional_find_by_username(&form.username, &gs.pool).await? {
        Some(user) => user,
        None => {
            err_template.form_error_message = Some(LoginUserError::IncorrectLogin);
            return html_with_status(err_template, status);
        }
    };

    if !verify_password(form.password.as_bytes(), user.password_hash.as_deref())? {
        err_template.form_error_message = Some(LoginUserError::IncorrectLogin);
        return html_with_status(err_template, status);
    }

    let jwt_cookie = generate_jwt_cookie(&gs.jwt_encoding_key, user.user_id)?;
    let refresh_token = RefreshToken::new_for_user(user.user_id, &mut txn).await?;
    let refresh_cookie = build_cookie(
        COOKIE_REFRESH,
        refresh_token.refresh_token.to_string(),
        Some(COOKIE_REFRESH_MAX_AGE),
    );

    txn.commit().await?;

    let redirect_url = match params.redirect {
        Some(redirect) => redirect,
        None => format!("/user/{}", user.username),
    };

    let jar = jar.add(jwt_cookie).add(refresh_cookie);
    Ok((jar, Redirect::to(&redirect_url)).into_response())
}

/// The route for `GET /account/logout`
pub async fn account_logout(State(gs): State<GlobalState>, jar: PrivateCookieJar) -> WebResult {
    // If there is a refresh token in the cookies then delete that token from the database
    if let Some(Ok(refresh)) = jar
        .get(COOKIE_REFRESH)
        .map(|refresh| Uuid::try_parse(refresh.value()))
    {
        RefreshToken::optional_delete(&refresh, &gs.pool).await?;
    }

    let jar = clear_cookie_jar(jar);
    Ok((jar, Redirect::to("/")).into_response())
}

/// The route for `POST /account/invites`
pub async fn do_account_invites(State(gs): State<GlobalState>, auth: MustAuth) -> WebResult {
    let user_id = auth.user.user_id;
    let user_invite_settings = UserInviteSettings::get_by_user_id(user_id, &gs.pool).await?;

    if let Some((invite, user_invite_settings)) =
        Invite::create_new_and_subtract_count(user_invite_settings, &gs.pool).await?
    {
        debug_assert_eq!(invite.user_creator_id, user_id);
        debug_assert_eq!(user_invite_settings.user_id, user_id);
        Ok(Redirect::to("/account/invites").into_response())
    } else {
        tracing::warn!(
            user_id = user_id,
            "user attempted to generate invite code without having available invite count"
        );
        html_with_status(
            AccountInvites {
                auth: auth.into_auth(),
                config: gs.config,
                settings: UserInviteSettings::get_by_user_id(user_id, &gs.pool).await?,
                invites: Invite::get_available_by_creator(user_id, &gs.pool).await?,
            },
            StatusCode::FORBIDDEN,
        )
    }
}
