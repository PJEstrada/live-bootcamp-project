use std::sync::Arc;
use axum::{http::StatusCode, response::IntoResponse, Json};
use axum::extract::State;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use log::log;
use crate::{app_state::AppState, domain::user::User};
use crate::domain::data_stores::{LoginAttemptId, TwoFaCode};
use crate::domain::email::Email;
use crate::domain::error::AuthAPIError;
use crate::domain::HashedPassword;
use crate::routes::SignupResponse;
use crate::utils::auth::generate_auth_cookie;

pub async fn login(State(state): State<AppState>,
                   jar: CookieJar,
                   Json(request): Json<LoginRequest>) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>){

    let email = match Email::parse(request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials))
    };

    let _ = match HashedPassword::parse(request.password.clone()).await {
        Ok(password) => password,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials))
    };

    let user_store = &state.user_store.read().await;

    if user_store.validate_user(&email, request.password.as_ref()).await.is_err() {
        return (jar, Err(AuthAPIError::IncorrectCredentials))
    }

    let user = match user_store.get_user(&email).await {
        Ok(user) => user,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    // Handle request based on user's 2FA configuration
    match user.requires_2fa {
        true => handle_2fa( &email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar).await,
    }
}

async fn handle_2fa(
    email: &Email, // New!
    state: &AppState, // New!
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    // First, we must generate a new random login attempt ID and 2FA code
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFaCode::default();
    let mut store = state.two_fa_code_store.write().await;

    if store.add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone()).await.is_err() {
        return (jar, Err(AuthAPIError::UnexpectedError))
    }

    // send 2FA code via the email client. Return `AuthAPIError::UnexpectedError` if the operation fails.
    let mail_client = state.email_client.write().await;
    let send_res = mail_client.send_email(
        email,
        "Your 2fa code",
        format!("Your 2fa code is: {:?}", two_fa_code.as_ref()).as_str()).await;
    if send_res.is_err() {
        return (jar, Err(AuthAPIError::UnexpectedError))
    }
    // Finally, we need to return the login attempt ID to the client
    let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_owned(),
        login_attempt_id: login_attempt_id.as_ref().to_string(), // Add the generated login attempt ID
    }));

    (jar, Ok((StatusCode::PARTIAL_CONTENT, response)))
}

// New!
async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    // Call the generate_auth_cookie function defined in the auth module.
    // If the function call fails return AuthAPIError::UnexpectedError.
    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError))
    };

    let updated_jar = jar.add(auth_cookie);


    (updated_jar, Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))))
}


#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// The login route can return 2 possible success responses.
// This enum models each response!
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}