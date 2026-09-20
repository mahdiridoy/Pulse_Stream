use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::session::AuthUser;
use crate::middleware::ssrf::validate_url;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct AddAddonRequest {
    pub name: String,
    pub manifest_url: String,
}

pub async fn list_addons(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let addons = state
        .repo
        .get_addons(auth.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to load addons."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": addons
    })))
}

pub async fn add_addon(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<AddAddonRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"success": false, "error": {"code": "VALIDATION_ERROR", "message": "Addon name is required."}}),
            ),
        ));
    }

    if let Err(e) = validate_url(&body.manifest_url) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"success": false, "error": {"code": "INVALID_URL", "message": e.to_string()}}),
            ),
        ));
    }

    let addon = state
        .repo
        .add_addon(auth.user_id, &body.name, &body.manifest_url)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to add addon."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": addon
    })))
}

pub async fn remove_addon(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    state
        .repo
        .remove_addon(auth.user_id, id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to remove addon."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": null
    })))
}

pub async fn toggle_addon(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let addon = state
        .repo
        .toggle_addon(auth.user_id, id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to toggle addon."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": addon
    })))
}
