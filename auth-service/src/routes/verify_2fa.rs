use axum::{http::StatusCode, response::IntoResponse, Json};
use axum::extract::State;
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use crate::app_state::AppState;
use crate::domain::data_stores::{LoginAttemptId, TwoFaCode};
use crate::domain::email::Email;
use crate::domain::error::AuthAPIError;
use crate::utils::auth::generate_auth_cookie;

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>){
    let email = match Email::parse(request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials))
    };

    let login_attempt_id = match LoginAttemptId::parse(request.login_attempt_id.clone()){
        Ok(attempt_id) => attempt_id,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials))
    };

    let two_fa_code = match TwoFaCode::parse( request.two_fa_code.clone()) {
        Ok(two_fa_code) => two_fa_code,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials))
    };

    let mut two_fa_code_store = state.two_fa_code_store.write().await;
    // Call `two_fa_code_store.get_code`. If the call fails
    // return a `AuthAPIError::IncorrectCredentials`.
    let code_tuple = match two_fa_code_store.get_code(&email).await {
        Ok(code_tuple) => code_tuple,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials))
    };

    if code_tuple.0.as_ref() != login_attempt_id.as_ref() || code_tuple.1.as_ref() != two_fa_code.as_ref()   {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    // Call the generate_auth_cookie function defined in the auth module.
    // If the function call fails return AuthAPIError::UnexpectedError.
    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError))
    };

    let updated_jar = jar.add(auth_cookie);

    // remove 2fa code from store
    match two_fa_code_store.remove_code(&email).await {
        Ok(_) =>  (updated_jar, Ok(StatusCode::OK.into_response())),
        Err(_) => (updated_jar, Err(AuthAPIError::IncorrectCredentials))
    }


}


#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String

}