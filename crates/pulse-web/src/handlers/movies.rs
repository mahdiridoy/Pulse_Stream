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

pub async fn list_movies(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    match state
        .service
        .search_typed(pulse_core::providers::ProviderKind::MovieBox, "", page)
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
                "message": "Unable to load movies."
            }
        })),
    }
}

pub async fn get_movie(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state
        .service
        .details_typed(pulse_core::providers::ProviderKind::MovieBox, &id)
        .await
    {
        Ok(details) => Json(json!({
            "success": true,
            "data": details,
            "error": null
        })),
        Err(_e) => Json(json!({
            "success": false,
            "data": null,
            "error": {
                "code": "PROVIDER_ERROR",
                "message": "Unable to load movie details."
            }
        })),
    }
}
