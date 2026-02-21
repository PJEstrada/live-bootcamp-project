use std::sync::{Arc};
use sqlx::PgPool;
use tokio::sync::RwLock;
use auth_service::{app_state::AppState, services::data_stores::postgres_user_store, services::data_stores::redis_banned_token_store::RedisBannedTokenStore, utils::constants::prod, Application, get_postgres_pool, get_redis_client};
use auth_service::services::data_stores::redis_two_fa_code_store::RedisTwoFACodeStore;
use auth_service::services::data_stores::postgres_user_store::PostgresUserStore;
use auth_service::services::mock_email_client::MockEmailClient;
use auth_service::utils::constants::{DATABASE_URL, REDIS_HOST_NAME};

#[tokio::main]
async fn main() {
    let pg_pool = configure_postgresql().await;
    let user_store =  Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let redis_conn = Arc::new(RwLock::new(configure_redis()));
    let banned_token_store =  Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_conn)));
    let redis_conn_2fa = Arc::new(RwLock::new(configure_redis()));
    let two_fa_token_store =  Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_conn_2fa)));
    let email_client =  Arc::new(RwLock::new(MockEmailClient{}));
    let app_state = AppState::new(user_store, banned_token_store, two_fa_token_store, email_client);

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");
    app.run().await.expect("Failed to run app");
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}