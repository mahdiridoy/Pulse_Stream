use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListParams {
    pub page: Option<usize>,
}

pub async fn list_anime(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    match state
        .service
        .search_typed(pulse_core::providers::ProviderKind::MovieBox, "anime", page)
        .await
    {
        Ok(items) => Json(json!({
            "success": true,
            "data": items,
            "error": null
        })),
        Err(_e) => Json(json!({
            "success": false,
            "data": null,
            "error": {
                "code": "PROVIDER_ERROR",
                "message": "Unable to load anime."
            }
        })),
    }
}
