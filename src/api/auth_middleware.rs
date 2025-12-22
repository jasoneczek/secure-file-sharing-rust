use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::auth::token::verify_token;

/// Authentication middleware
pub async fn auth_middleware(mut req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let claims = verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Make user_id available to handers
    req.extensions_mut().insert(claims.sub);

    Ok(next.run(req).await)
}
