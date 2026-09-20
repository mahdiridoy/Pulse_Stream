use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};

use crate::auth::session::AdminUser;
use crate::db::models::PublicUser;
use crate::state::AppState;

pub async fn admin_stats(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let users = state.repo.count_users().await.unwrap_or(0);
    let sessions = state.repo.count_sessions().await.unwrap_or(0);
    let favorites = state.repo.count_favorites().await.unwrap_or(0);

    Ok(Json(json!({
        "success": true,
        "data": {
            "total_users": users,
            "active_sessions": sessions,
            "total_favorites": favorites,
            "version": env!("CARGO_PKG_VERSION"),
        }
    })))
}

pub async fn admin_list_users(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let users: Vec<PublicUser> = state.repo.list_users().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to load users."}})),
        )
    })?;

    Ok(Json(json!({
        "success": true,
        "data": users
    })))
}

pub async fn admin_delete_user(
    _admin: AdminUser,
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    state.repo.delete_user(user_id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to delete user."}})),
        )
    })?;

    Ok(Json(json!({
        "success": true,
        "data": null
    })))
}
