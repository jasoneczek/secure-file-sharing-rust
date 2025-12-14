use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use crate::auth::service::AuthService;
use crate::auth::types::{RegisterRequest, LoginRequest, AuthTokenResponse};
use crate::api::AppState;

/// POST /register
pub async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthTokenResponse>, (StatusCode, String)> {
    let auth = state.auth.clone();

    match auth.register(req) {
        Ok(token) => Ok(Json(token)),
        Err(msg) => Err((StatusCode::BAD_REQUEST, msg)),
    }
}

/// POST /login
pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthTokenResponse>, (StatusCode, String)> {
    let auth = state.auth.clone();

    match auth.login(req) {
        Ok(token) => Ok(Json(token)),
        Err(msg) => Err((StatusCode::UNAUTHORIZED, msg)),
    }
}