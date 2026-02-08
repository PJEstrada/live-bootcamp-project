use axum::{http::StatusCode, response::IntoResponse};
use axum::extract::State;
use axum_extra::extract::CookieJar;

use crate::{

    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};
use crate::app_state::AppState;
use crate::domain::error::AuthAPIError;

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Retrieve JWT cookie from the `CookieJar`
    // Return AuthAPIError::MissingToken is the cookie is not found
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIError::MissingToken))
    };

    let token = cookie.value().to_owned();

    // TODO: Validate JWT token by calling `validate_token` from the auth service.
    // If the token is valid you can ignore the returned claims for now.
    // Return AuthAPIError::InvalidToken is validation fails.
    if validate_token(&token).await.is_err() {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    // Remove JWT cookie from the CookieJar
    let jar = jar.clone().remove(cookie.clone());

    if state.banned_token_store.write().await.add_token(token).await.is_err(){
        return (jar, Err(AuthAPIError::UnexpectedError));
    }

    (jar, Ok(StatusCode::OK))
}