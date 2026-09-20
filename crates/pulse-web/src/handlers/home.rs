use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::state::AppState;

#[derive(Serialize, Deserialize)]
struct CachedHomepage {
    items: Vec<serde_json::Value>,
}

pub async fn home(State(state): State<AppState>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let cache_key = "pulse:home:v1";

    if let Some(cached) = state.redis.get::<CachedHomepage>(cache_key).await {
        return Ok(Json(json!({
            "success": true,
            "data": cached.items,
            "error": null
        })));
    }

    match state.service.homepage("", 1).await {
        Ok((items, _metrics)) => {
            let items_value = serde_json::to_value(&items).unwrap_or_default();
            let items_vec = items_value.as_array().cloned().unwrap_or_default();

            let cached = CachedHomepage {
                items: items_vec.clone(),
            };
            state.redis.set(cache_key, &cached, 300).await;

            Ok(Json(json!({
                "success": true,
                "data": items,
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
                    "message": "Unable to load homepage."
                }
            })),
        )),
    }
}
