use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct StreamParams {
    pub season: Option<usize>,
    pub episode: Option<usize>,
}

pub async fn get_streams(
    State(state): State<AppState>,
    Path((provider, id)): Path<(String, String)>,
    Query(params): Query<StreamParams>,
) -> impl IntoResponse {
    let provider_kind = match provider.as_str() {
        "moviebox" => pulse_core::providers::ProviderKind::MovieBox,
        "fourkhdhub" => pulse_core::providers::ProviderKind::FourKHdHub,
        _ => {
            return Json(json!({
                "success": false,
                "data": null,
                "error": {
                    "code": "INVALID_PROVIDER",
                    "message": format!("Unknown provider: {}", provider)
                }
            }));
        }
    };

    let season = params.season.unwrap_or(1);
    let episode = params.episode.unwrap_or(1);

    match state
        .service
        .episode_streams(provider_kind, &id, season, episode)
        .await
    {
        Ok(releases) => Json(json!({
            "success": true,
            "data": releases,
            "error": null
        })),
        Err(_e) => Json(json!({
            "success": false,
            "data": null,
            "error": {
                "code": "PROVIDER_ERROR",
                "message": "Unable to load streams."
            }
        })),
    }
}
