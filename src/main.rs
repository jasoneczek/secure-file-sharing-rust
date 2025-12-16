mod models;
mod traits;
mod repository;
mod api;
mod auth;
mod storage;

use std::sync::Arc;
use parking_lot::Mutex;

use axum::{
    Router,
    routing::{get, post},
    middleware,
};

use tokio::net::TcpListener;

use api::{AppState, health_check};
use api::auth::{register_handler, login_handler};
use api::me::me_handler;
use api::auth_middleware::auth_middleware;

use repository::{UserRepository, FileRepository, PermissionRepository};
use auth::repository::AuthUserRepository;
use auth::service::SimpleAuthService;

#[tokio::main]
async fn main() {
    println!("\n=== File Sharing Server ===");

    // Build auth service
    let auth_repo = AuthUserRepository::new();
    let auth_service = SimpleAuthService::new(auth_repo);

    // Build application state
    let state = AppState {
        users: Arc::new(Mutex::new(UserRepository::new())),
        files: Arc::new(Mutex::new(FileRepository::new())),
        permissions: Arc::new(Mutex::new(PermissionRepository::new())),
        auth: auth_service,
    };

    let protected_state = state.clone();

    // Public routes
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/register", post(register_handler))
        .route("/login", post(login_handler));

    // Protected routes
    let protected_routes = Router::new()
        .route("/me", get(me_handler))
        .layer(middleware::from_fn_with_state(
            protected_state,
            auth_middleware,
        ));

    // Combine routers
    let app = public_routes
        .merge(protected_routes)
        .with_state(state);

    // Listener
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running at http://0.0.0.0:8080/");

    // Start server
    axum::serve(listener, app)
        .await
        .unwrap();
}