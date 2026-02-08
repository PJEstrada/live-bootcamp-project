use axum::extract::State;
use axum::Json;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use crate::app_state::AppState;
use crate::domain::error::AuthAPIError;
use crate::routes::LoginRequest;
use crate::utils::auth::validate_token;
use crate::utils::constants::JWT_COOKIE_NAME;

pub async fn verify_token(state: State<AppState>, jar: CookieJar, Json(request): Json<VerifyTokenRequest> ) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let token = request.token;

    if state.banned_token_store.read().await.is_token_banned(token.clone()).await {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    if validate_token(&token).await.is_err() {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    (jar, Ok(()))
}
#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    token: String
}