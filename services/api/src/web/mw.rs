use crate::config::Config;
use crate::error::AppError;
use crate::utils::jwt::decode_jwt;
use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub async fn auth_guard(
    State(state): State<Arc<Config>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_owned())
            } else {
                None
            }
        })
        .ok_or(AppError::Unauthorized)?;

    let claims = decode_jwt(&token, &state.jwt_secret)?;

    // Insert user_id into request extensions for handlers to use
    req.extensions_mut().insert(claims.sub);

    Ok(next.run(req).await)
}
