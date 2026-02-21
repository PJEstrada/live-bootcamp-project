use crate::helpers::TestApp;
use auth_service_macros::test_with_cleanup;

#[test_with_cleanup]
async fn root_returns_auth_ui() {
    let response = app.get_root().await;

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
}
