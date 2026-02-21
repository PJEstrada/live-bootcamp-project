use auth_service::{utils::constants::JWT_COOKIE_NAME};
use reqwest::Url;

use crate::helpers::{get_random_email, signup_and_login, TestApp};
use auth_service_macros::test_with_cleanup;

#[test_with_cleanup]
async fn should_return_400_if_jwt_cookie_missing() {
    let response = app.logout().await;
    assert_eq!(response.status().as_u16(), 400);
}

#[test_with_cleanup]
async fn should_return_401_if_invalid_token() {
    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.logout().await;
    assert_eq!(response.status().as_u16(), 401);
}

#[test_with_cleanup]
async fn should_return_200_if_valid_jwt_cookie() {

    let token = signup_and_login(&app).await;

    let response = app.logout().await;
    assert_eq!(response.status().as_u16(), 200);

    // check that token was added to banned token store
    assert!(app.banned_token_store.read().await.is_token_banned(token).await);
}

#[test_with_cleanup]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    signup_and_login(&app).await;

    let response = app.logout().await;
    assert_eq!(response.status().as_u16(), 200);
    let response2 = app.logout().await;
    assert_eq!(response2.status().as_u16(), 400);
}
