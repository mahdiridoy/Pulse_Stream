use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::state::AppState;

pub async fn get_details(
    State(state): State<AppState>,
    Path((provider, id)): Path<(String, String)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let provider_kind = match provider.as_str() {
        "moviebox" => pulse_core::providers::ProviderKind::MovieBox,
        "fourkhdhub" => pulse_core::providers::ProviderKind::FourKHdHub,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "data": null,
                    "error": {
                        "code": "INVALID_PROVIDER",
                        "message": "Unknown content provider."
                    }
                })),
            ));
        }
    };

    let cache_key = format!("pulse:details:v1:{}:{}", provider, id);

    if let Some(cached) = state.redis.get::<serde_json::Value>(&cache_key).await {
        return Ok(Json(json!({
            "success": true,
            "data": cached,
            "error": null
        })));
    }

    match state.service.details_typed(provider_kind, &id).await {
        Ok(details) => {
            state.redis.set(&cache_key, &details, 3600).await;
            Ok(Json(json!({
                "success": true,
                "data": details,
                "error": null
            })))
        }
        Err(_e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "error": {
                    "code": "PROVIDER_ERROR",
                    "message": "Unable to load content details."
                }
            })),
        )),
    }
}
