use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    Json,
};

use crate::api::AppState;
use crate::auth::service::{AuthError, AuthService};
use crate::auth::types::{AuthTokenResponse, LoginRequest, RegisterRequest};

fn map_auth_error(err: AuthError) -> (StatusCode, &'static str) {
    match err {
        AuthError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        AuthError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
        AuthError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
    }
}

/// POST /register
pub async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthTokenResponse>, (StatusCode, &'static str)> {
    let auth = state.auth.clone();

    match auth.register(req).await {
        Ok(token) => Ok(Json(token)),
        Err(err) => Err(map_auth_error(err)),
    }
}

/// POST /login
pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthTokenResponse>, (StatusCode, &'static str)> {
    let auth = state.auth.clone();

    match auth.login(req).await {
        Ok(token) => Ok(Json(token)),
        Err(err) => Err(map_auth_error(err)),
    }
}

/// GET /token/refresh
pub async fn refresh_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthTokenResponse>, (StatusCode, &'static str)> {
    let auth = state.auth.clone();

    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header"))?;

    let refresh_token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid Authorization header"))?
        .to_string(); // This one is fine: refresh_token must be owned

    match auth.refresh(refresh_token).await {
        Ok(token) => Ok(Json(token)),
        Err(err) => Err(map_auth_error(err)),
    }
}


