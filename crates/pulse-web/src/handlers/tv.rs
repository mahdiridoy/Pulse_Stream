use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;

pub async fn list_channels() -> impl IntoResponse {
    Json(json!({
        "success": true,
        "data": [],
        "error": null
    }))
}
