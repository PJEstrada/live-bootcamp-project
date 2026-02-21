use auth_service::domain::error::ErrorResponse;
use crate::helpers::{get_random_email, TestApp};
use auth_service_macros::test_with_cleanup;

#[test_with_cleanup]
async fn should_return_201_if_valid_input() {
    let random_email = get_random_email();
    let body =         serde_json::json!({
            "email": random_email,
            "password": "mysecretpassword123",
            "requires2FA": true,
        });
    let response = app.post_signup(&body).await;
    assert_eq!(
        response.status().as_u16(),
        201,
        "Failed for input: {:?}",
        body
    );
}

#[test_with_cleanup]
async fn should_return_422_if_malformed_input() {
    let random_email = get_random_email();

    // TODO: add more malformed input test cases
    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "password": "password123",
            "bad": true
        }),
        serde_json::json!({
            "password": "password123",
            "email": true
        }),
        serde_json::json!({
            "password": false,
            "email": "password123"
        }),
        serde_json::json!({
            "requires2FA": "password123",
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
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
    // The signup route should return a 400 HTTP status code if an invalid input is sent.
    // The input is considered invalid if:
    // - The email is empty or does not contain '@'
    // - The password is less than 8 characters
    // TODO: add more malformed input test cases
    let test_cases = [
        serde_json::json!({
            "password": "123456",
            "email": "pablo@gmail.com",
            "requires2FA": true
        }),
        serde_json::json!({
            "password": "1234567898",
            "email": "pablogmail.com",
            "requires2FA": true
        }),

    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(response.status().as_u16(), 400, "Failed for input: {:?}", test_cases);

        assert_eq!(
            response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
            "Invalid credentials".to_owned()
        );
    }
}

#[test_with_cleanup]
async fn should_return_409_if_email_already_exists() {

    let payload = serde_json::json!({
            "password": "123456789",
            "email": "pablo@gmail.com",
            "requires2FA": true
        });
    let response1 = app.post_signup(&payload).await;
    assert_eq!(
        response1.status().as_u16(),
        201,
        "Failed for input: {:?}",
        payload
    );
    let response2 = app.post_signup(&payload).await;
    assert_eq!(response2.status().as_u16(), 409);

    assert_eq!(
        response2
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
}