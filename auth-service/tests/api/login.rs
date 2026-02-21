use crate::helpers::{get_random_email, TestApp};
use auth_service::{utils::constants::JWT_COOKIE_NAME};
use auth_service::domain::email::Email;
use auth_service::routes::TwoFactorAuthResponse;
use auth_service_macros::test_with_cleanup;

#[test_with_cleanup]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let random_email = get_random_email();
    let mail = Email::parse(random_email.clone()).unwrap();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());


    // TODO: assert that `json_body.login_attempt_id` is stored inside `app.two_fa_code_store`
    let result = app.two_fa_code_store.read().await.get_code(&mail).await.unwrap();
    assert_eq!(result.0.as_ref(), json_body.login_attempt_id);

}

#[test_with_cleanup]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}


#[test_with_cleanup]
async fn should_return_422_if_malformed_input() {
    let random_email = get_random_email();

    // TODO: add more malformed input test cases
    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "email2": true
        }),
        serde_json::json!({
            "bad2": "password123",
            "bad": " 123"
        }),
        serde_json::json!({
            "password": "password123",
            "email": true
        }),
        serde_json::json!({
            "somefield": "password123",
        }),
        serde_json::json!({
            "password": 123,
            "email": "test@test.com"
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[test_with_cleanup]
async fn should_return_400_if_invalid_input() {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.

    let test_cases = [
        serde_json::json!({
            "password": "small",
            "email": "testtest.com"
        }),
        serde_json::json!({
            "password": "loooonnng123",
            "email": "testtest.com"
        }),
        serde_json::json!({
            "password": "123",
            "email": "testtest@test.com"
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[test_with_cleanup]
async fn should_return_401_if_incorrect_credentials() {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.
    let body = serde_json::json!({
        "email": "test@test.com",
        "password": "avalidpassword123"
    });
    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 401);
}