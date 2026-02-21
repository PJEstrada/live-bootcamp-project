use std::sync::Arc;
use axum::{http::StatusCode, response::IntoResponse, Json};
use axum::extract::State;
use serde::{Deserialize, Serialize};
use crate::{app_state::AppState, domain::user::User};
use crate::domain::email::Email;
use crate::domain::error::AuthAPIError;
use crate::domain::HashedPassword;

pub async fn signup(State(state): State<AppState>,
                    Json(request): Json<SignupRequest>) -> Result<impl IntoResponse, AuthAPIError>{

    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password = HashedPassword::parse( request.password).await.map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user = User::new(email, password, request.requires_2fa);
    let mut user_store = state.user_store.write().await;


    if user_store.get_user(&user.email).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }
    if user_store.add_user(user).await.is_err() {
        return Err(AuthAPIError::UnexpectedError);
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}
#[derive(Serialize)]
pub struct SignupResponse {
    pub message: String,
}