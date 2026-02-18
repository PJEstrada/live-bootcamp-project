use std::error::Error;
use std::sync::Arc;
use axum::http::{Method, StatusCode};
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::{post};
use axum::serve::Serve;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, services::ServeDir};
pub mod routes;

pub mod domain;
pub mod services;
pub mod app_state;
pub mod utils;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: String,
}

impl Application {
    pub async fn build(app_state: app_state::AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let assets_dir = ServeDir::new("assets");

        let allowed_origins = [
            "http://localhost:8000".parse()?,
            "http://167.71.176.216:8000".parse()?,
        ];

        let cors = CorsLayer::new()
            // Allow GET and POST requests
            .allow_methods([Method::GET, Method::POST])
            // Allow cookies to be included in requests
            .allow_credentials(true)
            .allow_origin(allowed_origins);
        // Move the Router definition from `main.rs` to here.
        // Also, remove the `hello` route.
        // We don't need it at this point!
        let router =     Router::new()
            .fallback_service(assets_dir)
            .route("/signup", post(self::routes::signup))
            .route("/login", post(self::routes::login))
            .route("/verify-2fa", post(self::routes::verify_2fa))
            .route("/logout", post(self::routes::logout))
            .route("/verify-token", post(self::routes::verify_token))
            .with_state(app_state)
            .layer(cors);





        let listener = TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();

        let server = axum::serve(listener, router);
        // Create a new Application instance and return it
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}


async fn login() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

async fn verify_2fa() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

async fn logout() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

async fn verify_token() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
