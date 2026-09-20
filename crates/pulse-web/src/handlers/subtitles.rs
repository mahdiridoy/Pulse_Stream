use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use serde_json::json;

use crate::state::AppState;

pub async fn get_subtitles(
    State(_state): State<AppState>,
    Path((provider, id)): Path<(String, String)>,
) -> impl IntoResponse {
    let _ = provider;
    let _ = &id;

    Json(json!({
        "success": true,
        "data": [],
        "error": null
    }))
}
