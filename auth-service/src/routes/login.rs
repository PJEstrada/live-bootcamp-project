use std::sync::Arc;
use axum::{http::StatusCode, response::IntoResponse, Json};
use axum::extract::State;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use crate::{app_state::AppState, domain::user::User};
use crate::domain::email::Email;
use crate::domain::error::AuthAPIError;
use crate::domain::Password;
use crate::utils::auth::generate_auth_cookie;

pub async fn login(State(state): State<AppState>,
                   jar: CookieJar,
                   Json(request): Json<LoginRequest>) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>){

    let email = match Email::parse(request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials))
    };
    let password = match Password::parse( request.password) {
        Ok(password) => password,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials))
    };

    let user_store = &state.user_store.read().await;

    if user_store.validate_user(&email, &password).await.is_err() {
        return (jar, Err(AuthAPIError::IncorrectCredentials))
    }

    let user = match user_store.get_user(&email).await {
        Ok(user) => user,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError))
    };

    // Call the generate_auth_cookie function defined in the auth module.
    // If the function call fails return AuthAPIError::UnexpectedError.
    let auth_cookie = match generate_auth_cookie(&user.email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError))
    };

    let updated_jar = jar.add(auth_cookie);

    (updated_jar, Ok(StatusCode::OK.into_response()))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
