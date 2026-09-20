use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::session::AuthUser;
use crate::middleware::ssrf::validate_url;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct AddPlaylistRequest {
    pub name: String,
    pub url: String,
}

pub async fn list_playlists(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let playlists = state
        .repo
        .get_playlists(auth.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to load playlists."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": playlists
    })))
}

pub async fn add_playlist(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<AddPlaylistRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"success": false, "error": {"code": "VALIDATION_ERROR", "message": "Playlist name is required."}}),
            ),
        ));
    }

    if let Err(e) = validate_url(&body.url) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"success": false, "error": {"code": "INVALID_URL", "message": e.to_string()}}),
            ),
        ));
    }

    let playlist = state
        .repo
        .add_playlist(auth.user_id, &body.name, &body.url)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to add playlist."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": playlist
    })))
}

pub async fn remove_playlist(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    state
        .repo
        .remove_playlist(auth.user_id, id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to remove playlist."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": null
    })))
}

pub async fn toggle_playlist(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let playlist = state
        .repo
        .toggle_playlist(auth.user_id, id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"success": false, "error": {"code": "SERVER_ERROR", "message": "Unable to toggle playlist."}})),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "data": playlist
    })))
}
