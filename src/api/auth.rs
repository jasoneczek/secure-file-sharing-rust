use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
};

use crate::api::AppState;
use crate::auth::service::AuthService;
use crate::auth::types::{AuthTokenResponse, LoginRequest, RegisterRequest};

/// POST /register
pub async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthTokenResponse>, (StatusCode, String)> {
    let auth = state.auth.clone();

    match auth.register(req).await {
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

    match auth.login(req).await {
        Ok(token) => Ok(Json(token)),
        Err(msg) => Err((StatusCode::UNAUTHORIZED, msg)),
    }
}

/// GET /token/refresh
pub async fn refresh_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthTokenResponse>, (StatusCode, String)> {
    let auth = state.auth.clone();

    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing Authorization header".into(),
        ))?;

    let refresh_token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid Authorization header".into(),
        ))?
        .to_string();

    match auth.refresh(refresh_token).await {
        Ok(token) => Ok(Json(token)),
        Err(msg) => Err((StatusCode::UNAUTHORIZED, msg)),
    }
}
