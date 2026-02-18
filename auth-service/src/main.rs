use std::sync::{Arc};
use tokio::sync::RwLock;
use auth_service::{
    app_state::AppState,
    services::hashmap_user_store::HashmapUserStore,
    services::banned_token_store::HashsetBannedTokenStore,
    utils::constants::prod,
    Application,
};
use auth_service::app_state::TwoFaCodeStoreType;
use auth_service::domain::data_stores::TwoFACodeStoreError;
use auth_service::services::hashmap_two_fa_code_store::HashmapTwoFACodeStore;
use auth_service::services::mock_email_client::MockEmailClient;

#[tokio::main]
async fn main() {
    let user_store =  Arc::new(RwLock::new(HashmapUserStore::default()));
    let banned_token_store =  Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let two_fa_token_store =  Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
    let email_client =  Arc::new(RwLock::new(MockEmailClient{}));
    let app_state = AppState::new(user_store, banned_token_store, two_fa_token_store, email_client);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");
    app.run().await.expect("Failed to run app");
}