use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let state = parts.extensions.get::<AppState>().cloned().ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": {"code": "INTERNAL_ERROR", "message": "Server configuration error"}
                })),
            )
        })?;

        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        let cookie_token = parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| {
                v.split(';').find_map(|c| {
                    let c = c.trim();
                    c.strip_prefix("pulse_token=").map(|t| t.trim().to_string())
                })
            });

        let token = auth_header
            .map(|s| s.to_string())
            .or(cookie_token)
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::json!({
                        "success": false,
                        "error": {"code": "UNAUTHORIZED", "message": "Authentication required"}
                    })),
                )
            })?;

        let session = state.repo.find_session_by_token(&token).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": {"code": "INTERNAL_ERROR", "message": "Authentication service unavailable"}
                })),
            )
        })?;

        match session {
            Some(s) => Ok(AuthUser { user_id: s.user_id }),
            None => Err((
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": {"code": "UNAUTHORIZED", "message": "Invalid or expired session"}
                })),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdminUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let state = parts.extensions.get::<AppState>().cloned().ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": {"code": "INTERNAL_ERROR", "message": "Server configuration error"}
                })),
            )
        })?;

        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        let cookie_token = parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| {
                v.split(';').find_map(|c| {
                    let c = c.trim();
                    c.strip_prefix("pulse_token=").map(|t| t.trim().to_string())
                })
            });

        let token = auth_header
            .map(|s| s.to_string())
            .or(cookie_token)
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::json!({
                        "success": false,
                        "error": {"code": "UNAUTHORIZED", "message": "Authentication required"}
                    })),
                )
            })?;

        let session = state.repo.find_session_by_token(&token).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": {"code": "INTERNAL_ERROR", "message": "Authentication service unavailable"}
                })),
            )
        })?;

        let session = session.ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": {"code": "UNAUTHORIZED", "message": "Invalid or expired session"}
                })),
            )
        })?;

        let user = state.repo.find_user_by_id(session.user_id).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": {"code": "INTERNAL_ERROR", "message": "Authentication service unavailable"}
                })),
            )
        })?;

        match user {
            Some(u) if u.role == "admin" => Ok(AdminUser { user_id: u.id }),
            Some(_) => Err((
                StatusCode::FORBIDDEN,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": {"code": "FORBIDDEN", "message": "Admin access required"}
                })),
            )),
            None => Err((
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": {"code": "UNAUTHORIZED", "message": "User not found"}
                })),
            )),
        }
    }
}
