use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;

pub async fn list_providers() -> impl IntoResponse {
    Json(json!({
        "success": true,
        "data": [
            {
                "id": "moviebox",
                "name": "MovieBox",
                "supports_search": true,
                "supports_series": true,
                "supports_subtitles": true,
                "supports_homepage": true
            },
            {
                "id": "fourkhdhub",
                "name": "4KHDHub",
                "supports_search": true,
                "supports_series": true,
                "supports_subtitles": true,
                "supports_homepage": false
            }
        ],
        "error": null
    }))
}
