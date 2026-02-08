use crate::helpers::{get_random_email, TestApp};
use auth_service::{utils::constants::JWT_COOKIE_NAME};

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;

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

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

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
#[tokio::test]
async fn should_return_400_if_invalid_input() {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;

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

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.
    let app = TestApp::new().await;
    let body = serde_json::json!({
        "email": "test@test.com",
        "password": "avalidpassword123"
    });
    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 401);
}