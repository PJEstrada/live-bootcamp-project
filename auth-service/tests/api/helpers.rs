use std::sync::{Arc};
use reqwest::cookie::Jar;
use tokio::sync::RwLock;
use uuid::Uuid;
use auth_service::Application;
use auth_service::app_state::{AppState, BannedTokenStoreType, EmailClientType, TwoFaCodeStoreType, UserStoreType};
use auth_service::services::banned_token_store::HashsetBannedTokenStore;
use auth_service::services::hashmap_two_fa_code_store::HashmapTwoFACodeStore;
use auth_service::services::hashmap_user_store::HashmapUserStore;
use auth_service::services::mock_email_client::MockEmailClient;
use auth_service::utils::constants::{test, JWT_COOKIE_NAME};

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFaCodeStoreType,
    pub email_client: EmailClientType
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
        let banned_token_store =  Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
        let two_fa_code_store =  Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
        let email_client =  Arc::new(RwLock::new(MockEmailClient{}));
        let app_state = AppState::new(user_store, banned_token_store.clone(), two_fa_code_store.clone(), email_client.clone());
        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build app");
        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task to avoid blocking the main thread.
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::new();
        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();
        Self {
            address,
            cookie_jar,
            http_client,
            banned_token_store,
            two_fa_code_store,
            email_client,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client.get(&format!("{}/", self.address)).send().await.unwrap()
    }



    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn signup(&self) -> reqwest::Response {
        self.http_client.post(&format!("{}/signup", self.address)).send().await.unwrap()
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
        where Body: serde::Serialize, {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    
    pub async fn verify_2fa(&self) -> reqwest::Response {
        self.http_client.post(&format!("{}/verify-2fa", self.address)).send().await.unwrap()
    }

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client.post(&format!("{}/logout", self.address)).send().await.unwrap()
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }


}

pub  fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

pub async fn signup_and_login(app: &TestApp) -> String {
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
    auth_cookie.value().to_owned()
}