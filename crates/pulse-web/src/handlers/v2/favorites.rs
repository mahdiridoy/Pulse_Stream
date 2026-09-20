use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::session::AuthUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct AddFavoriteRequest {
    pub content_id: String,
    pub provider: String,
    pub media_type: String,
    pub title: Option<String>,
    pub poster_url: Option<String>,
}

pub async fn list_favorites(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let favorites = state
        .repo
        .get_favorites(auth.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to load favorites."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": favorites
    })))
}

pub async fn add_favorite(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<AddFavoriteRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.content_id.is_empty() || body.provider.is_empty() || body.media_type.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"success": false, "error": {"code": "VALIDATION_ERROR", "message": "content_id, provider, and media_type are required."}}),
            ),
        ));
    }

    let favorite = state
        .repo
        .add_favorite(
            auth.user_id,
            &body.content_id,
            &body.provider,
            &body.media_type,
            body.title.as_deref(),
            body.poster_url.as_deref(),
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to add favorite."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": favorite
    })))
}

pub async fn remove_favorite(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    state
        .repo
        .remove_favorite(auth.user_id, id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to remove favorite."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": null
    })))
}
