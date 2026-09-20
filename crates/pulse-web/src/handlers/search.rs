use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub provider: Option<String>,
    pub page: Option<usize>,
}

pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let page = params.page.unwrap_or(1);
    let provider_filter = params.provider.as_deref();
    let cache_key = format!(
        "pulse:search:v1:{}:{}:{}",
        params.q,
        provider_filter.unwrap_or("all"),
        page
    );

    if let Some(cached) = state.redis.get::<Vec<serde_json::Value>>(&cache_key).await {
        return Ok(Json(json!({
            "success": true,
            "data": cached,
            "error": null
        })));
    }

    let mut results = Vec::new();

    let providers_to_search: Vec<_> = if let Some(p) = provider_filter {
        match p {
            "moviebox" => vec![pulse_core::providers::ProviderKind::MovieBox],
            "fourkhdhub" => vec![pulse_core::providers::ProviderKind::FourKHdHub],
            _ => vec![],
        }
    } else {
        vec![
            pulse_core::providers::ProviderKind::MovieBox,
            pulse_core::providers::ProviderKind::FourKHdHub,
        ]
    };

    for provider_kind in providers_to_search {
        match state
            .service
            .search_typed(provider_kind, &params.q, page)
            .await
        {
            Ok(items) => results.extend(items),
            Err(e) => {
                log::warn!("Search failed for {:?}: {}", provider_kind, e);
            }
        }
    }

    if !results.is_empty() {
        state.redis.set(&cache_key, &results, 3600).await;
    }

    Ok(Json(json!({
        "success": true,
        "data": results,
        "error": null
    })))
}
