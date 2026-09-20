pub(super) use pulse_core::service::decode_poster;

pub(super) async fn fetch_poster_bytes(client: &reqwest::Client, url: &str) -> Option<Vec<u8>> {
    let response = client
        .get(url)
        .header("User-Agent", pulse_core::net::APP_HTTP_USER_AGENT)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    Some(response.bytes().await.ok()?.to_vec())
}
