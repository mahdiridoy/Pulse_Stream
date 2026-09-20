use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::session::AuthUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct UpdateProgressRequest {
    pub content_id: String,
    pub provider: String,
    pub media_type: String,
    pub title: Option<String>,
    pub poster_url: Option<String>,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub position: f64,
    pub duration: f64,
}

pub async fn list_history(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let history = state
        .repo
        .get_watch_history(auth.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to load watch history."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": history
    })))
}

pub async fn update_progress(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateProgressRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.content_id.is_empty() || body.provider.is_empty() || body.media_type.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"success": false, "error": {"code": "VALIDATION_ERROR", "message": "content_id, provider, and media_type are required."}}),
            ),
        ));
    }

    let entry = state
        .repo
        .upsert_watch_progress(
            auth.user_id,
            &body.content_id,
            &body.provider,
            &body.media_type,
            body.title.as_deref(),
            body.poster_url.as_deref(),
            body.season,
            body.episode,
            body.position,
            body.duration,
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to update progress."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": entry
    })))
}

pub async fn remove_history(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    state
        .repo
        .remove_watch_history(auth.user_id, id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to remove history entry."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": null
    })))
}

pub async fn continue_watching(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let items = state
        .repo
        .get_continue_watching(auth.user_id, 20)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to load continue watching."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": items
    })))
}
