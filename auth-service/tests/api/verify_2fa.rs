use auth_service::domain::data_stores::LoginAttemptId;
use auth_service::domain::email::Email;
use auth_service::routes::TwoFactorAuthResponse;
use auth_service::utils::constants::JWT_COOKIE_NAME;
use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({
            "email": "password123",
            "loginAttemptId": "123",
            "2FACode": false
        }),
        serde_json::json!({
            "bad2": "password123",
            "bad": " 123"
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(test_case).await;
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
    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({
            "email": "password123",
            "loginAttemptId": "123",
            "2FACode": "false"
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(test_case).await;
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
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let attempt = LoginAttemptId::default();
    let verify_2fa_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": attempt.as_ref(),
        "2FACode": "123456"
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 401);


}

#[tokio::test]
async fn should_return_401_if_old_code() {
    // Call login twice. Then, attempt to call verify-fa with the 2FA code from the first login requet. This should fail.
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let mail = Email::parse(random_email.clone()).unwrap();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);
    let login_1_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "2FACode": "123"
    });

    let response = app.post_login(&login_1_body).await;
    assert_eq!(response.status().as_u16(), 206);

    let result_1 = app.two_fa_code_store.read().await.get_code(&mail).await.unwrap();
    let attempt_1 = result_1.0;
    let code_1 = result_1.1;
    // login again
    let response = app.post_login(&login_1_body).await;

    let verify_2fa_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": attempt_1.as_ref(),
        "2FACode": code_1.as_ref()
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 401);
}


#[tokio::test]
async fn should_return_200_if_correct_code() {
    let app = TestApp::new().await;

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

    let result = app.two_fa_code_store.read().await.get_code(&mail).await.unwrap();
    assert_eq!(result.0.as_ref(), json_body.login_attempt_id);

    let verify_2fa_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": result.0.as_ref(),
        "2FACode": result.1.as_ref()
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

}


#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let app = TestApp::new().await;

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

    let result = app.two_fa_code_store.read().await.get_code(&mail).await.unwrap();
    assert_eq!(result.0.as_ref(), json_body.login_attempt_id);

    let verify_2fa_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": result.0.as_ref(),
        "2FACode": result.1.as_ref()
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    let verify_2fa_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": result.0.as_ref(),
        "2FACode": result.1.as_ref()
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 401);

}