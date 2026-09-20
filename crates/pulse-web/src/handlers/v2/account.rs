use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};

use crate::auth::session::AuthUser;
use crate::state::AppState;

pub async fn get_account(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user = state
        .repo
        .find_user_by_id(auth.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to load profile."}})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"success": false, "error": {"code": "NOT_FOUND", "message": "User not found."}})),
            )
        })?;

    let favorites_count: i64 = state
        .repo
        .get_favorites(auth.user_id)
        .await
        .map(|f| f.len() as i64)
        .unwrap_or(0);

    let history_count: i64 = state
        .repo
        .get_watch_history(auth.user_id)
        .await
        .map(|h| h.len() as i64)
        .unwrap_or(0);

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": user.id,
            "email": user.email,
            "username": user.username,
            "role": user.role,
            "favorites_count": favorites_count,
            "history_count": history_count,
            "created_at": user.created_at
        }
    })))
}
