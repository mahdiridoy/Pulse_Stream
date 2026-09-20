use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};

use crate::middleware::ssrf::{ssrf_protected_client, validate_url};
use crate::state::AppState;

pub async fn list_channels(
    State(_state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let playlist_url = std::env::var("IPTV_PLAYLIST_URL").unwrap_or_else(|_| String::new());

    if playlist_url.is_empty() {
        return Ok(Json(json!({
            "success": true,
            "data": [],
            "message": "No IPTV playlist configured. Set IPTV_PLAYLIST_URL environment variable."
        })));
    }

    // Validate URL to prevent SSRF (static check)
    if let Err(e) = validate_url(&playlist_url) {
        tracing::warn!("IPTV playlist URL blocked by SSRF protection: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"success": false, "error": {"code": "INVALID_URL", "message": "Invalid IPTV playlist URL."}}),
            ),
        ));
    }

    // Use SSRF-protected client for the actual connection (DNS rebinding protection)
    let client = ssrf_protected_client().await;
    let parser = pulse_core::providers::tv::M3UParser::with_client(client);

    match parser.fetch_playlist(&playlist_url).await {
        Ok(channels) => {
            let data: Vec<Value> = channels
                .iter()
                .map(|ch| {
                    json!({
                        "id": ch.id,
                        "name": ch.name,
                        "logo": ch.logo,
                        "group": ch.group,
                        "stream_url": ch.stream_url,
                    })
                })
                .collect();

            Ok(Json(json!({
                "success": true,
                "data": data
            })))
        }
        Err(e) => {
            tracing::warn!("Failed to fetch IPTV playlist: {}", e);
            Err((
                StatusCode::BAD_GATEWAY,
                Json(
                    json!({"success": false, "error": {"code": "PLAYLIST_ERROR", "message": "Unable to load IPTV playlist."}}),
                ),
            ))
        }
    }
}

pub async fn get_channel_stream(
    State(_state): State<AppState>,
    axum::extract::Path(channel_id): axum::extract::Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let playlist_url = std::env::var("IPTV_PLAYLIST_URL").unwrap_or_else(|_| String::new());

    if playlist_url.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(
                json!({"success": false, "error": {"code": "NOT_FOUND", "message": "No IPTV playlist configured."}}),
            ),
        ));
    }

    // Validate URL to prevent SSRF (static check)
    if let Err(e) = validate_url(&playlist_url) {
        tracing::warn!("IPTV playlist URL blocked by SSRF protection: {}", e);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"success": false, "error": {"code": "INVALID_URL", "message": "Invalid IPTV playlist URL."}}),
            ),
        ));
    }

    // Use SSRF-protected client for the actual connection (DNS rebinding protection)
    let client = ssrf_protected_client().await;
    let parser = pulse_core::providers::tv::M3UParser::with_client(client);

    if let Ok(channels) = parser.fetch_playlist(&playlist_url).await {
        if let Some(ch) = channels
            .iter()
            .find(|c| c.id == channel_id || c.name == channel_id)
        {
            return Ok(Json(json!({
                "success": true,
                "data": {
                    "url": ch.stream_url,
                    "name": ch.name,
                    "logo": ch.logo,
                }
            })));
        }
    }

    Err((
        StatusCode::NOT_FOUND,
        Json(
            json!({"success": false, "error": {"code": "NOT_FOUND", "message": "Channel not found."}}),
        ),
    ))
}
