use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    response::Response,
};
use std::collections::HashMap;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower::{Layer, Service};

// ---------------------------------------------------------------------------
// Proxy configuration
// ---------------------------------------------------------------------------

/// Configuration for trusted-proxy-aware IP extraction.
///
/// Default: `trust_proxy = false` — always use the direct socket peer address.
/// When `trust_proxy = true`, the *immediate peer* must be in
/// `trusted_proxy_ips` for `X-Forwarded-For` to be honoured.
#[derive(Clone, Debug, Default)]
pub struct ProxyConfig {
    /// Whether to honour X-Forwarded-For from trusted peers.
    pub trust_proxy: bool,
    /// IP addresses / CIDRs of trusted proxies.
    /// When empty and `trust_proxy` is true, ALL direct peers are treated as
    /// trusted (less secure but simpler for single-proxy deployments).
    pub trusted_proxy_ips: Vec<IpAddr>,
}

impl ProxyConfig {
    /// Build from environment variables `TRUST_PROXY` and `TRUSTED_PROXY_IPS`.
    pub fn from_env() -> Self {
        let trust_proxy = std::env::var("TRUST_PROXY")
            .map(|v| v == "true" || v == "1" || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(false);

        let trusted_proxy_ips = std::env::var("TRUSTED_PROXY_IPS")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(|v| {
                v.split(',')
                    .filter_map(|s| IpAddr::from_str(s.trim()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Self {
            trust_proxy,
            trusted_proxy_ips,
        }
    }

    fn is_trusted_peer(&self, peer_ip: IpAddr) -> bool {
        if !self.trust_proxy {
            return false;
        }
        // If no explicit list, treat all peers as trusted (single-proxy mode)
        if self.trusted_proxy_ips.is_empty() {
            return true;
        }
        self.trusted_proxy_ips.contains(&peer_ip)
    }
}

// ---------------------------------------------------------------------------
// IP extraction helpers
// ---------------------------------------------------------------------------

/// Extract the "real" client IP from the request.
///
/// 1. Always start with the direct socket peer address.
/// 2. Only if `proxy_config.is_trusted_peer(peer_ip)` AND the request carries
///    a valid `X-Forwarded-For` header, take the *leftmost* (client) entry.
/// 3. Never blindly trust the header — the direct peer must be an authorised
///    proxy.
fn extract_client_ip<B>(req: &Request<B>, proxy_config: &ProxyConfig) -> String {
    let peer_ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip());

    let peer = match peer_ip {
        Some(ip) => ip,
        None => return "unknown".to_string(),
    };

    // Only look at X-Forwarded-For when the immediate peer is trusted
    if !proxy_config.is_trusted_peer(peer) {
        return peer.to_string();
    }

    // Parse X-Forwarded-For: "client, proxy1, proxy2, …"
    if let Some(xff) = req.headers().get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            if let Some(first) = xff_str.split(',').next() {
                let trimmed = first.trim();
                if let Ok(ip) = IpAddr::from_str(trimmed) {
                    return ip.to_string();
                }
            }
        }
    }

    // Fallback to peer
    peer.to_string()
}

// ---------------------------------------------------------------------------
// Rate limit entry
// ---------------------------------------------------------------------------

struct RateEntry {
    count: u32,
    window_start: Instant,
}

// ---------------------------------------------------------------------------
// Global rate limit layer / service
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct RateLimitLayer {
    pub max_requests: u32,
    pub window: Duration,
    pub proxy_config: ProxyConfig,
}

impl RateLimitLayer {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            proxy_config: ProxyConfig::default(),
        }
    }

    pub fn with_proxy_config(
        max_requests: u32,
        window: Duration,
        proxy_config: ProxyConfig,
    ) -> Self {
        Self {
            max_requests,
            window,
            proxy_config,
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            max_requests: self.max_requests,
            window: self.window,
            proxy_config: self.proxy_config.clone(),
            clients: Arc::new(RwLock::new(HashMap::new())),
            last_cleanup: Arc::new(tokio::sync::RwLock::new(Instant::now())),
        }
    }
}

// ---------------------------------------------------------------------------
// Path-based rate limit layer / service
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct PathRateLimitLayer {
    pub max_requests: u32,
    pub window: Duration,
    pub paths: Vec<String>,
    pub proxy_config: ProxyConfig,
}

impl PathRateLimitLayer {
    pub fn new(max_requests: u32, window: Duration, paths: Vec<String>) -> Self {
        Self {
            max_requests,
            window,
            paths,
            proxy_config: ProxyConfig::default(),
        }
    }

    pub fn with_proxy_config(
        max_requests: u32,
        window: Duration,
        paths: Vec<String>,
        proxy_config: ProxyConfig,
    ) -> Self {
        Self {
            max_requests,
            window,
            paths,
            proxy_config,
        }
    }
}

impl<S> Layer<S> for PathRateLimitLayer {
    type Service = PathRateLimitService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        PathRateLimitService {
            inner,
            max_requests: self.max_requests,
            window: self.window,
            paths: self.paths.clone(),
            proxy_config: self.proxy_config.clone(),
            clients: Arc::new(RwLock::new(HashMap::new())),
            last_cleanup: Arc::new(tokio::sync::RwLock::new(Instant::now())),
        }
    }
}

// ---------------------------------------------------------------------------
// Global rate limit service
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    max_requests: u32,
    window: Duration,
    proxy_config: ProxyConfig,
    clients: Arc<RwLock<HashMap<String, RateEntry>>>,
    last_cleanup: Arc<tokio::sync::RwLock<Instant>>,
}

impl<S, ReqBody> Service<Request<ReqBody>> for RateLimitService<S>
where
    S: Service<Request<ReqBody>, Response = Response> + Send + Clone + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let clients = self.clients.clone();
        let max_requests = self.max_requests;
        let window = self.window;
        let last_cleanup = self.last_cleanup.clone();
        let proxy_config = self.proxy_config.clone();

        Box::pin(async move {
            let ip = extract_client_ip(&req, &proxy_config);

            let now = Instant::now();

            // Periodic cleanup every 5 minutes
            {
                let mut last = last_cleanup.write().await;
                if now.duration_since(*last) > Duration::from_secs(300) {
                    let mut clients_guard = clients.write().await;
                    clients_guard
                        .retain(|_, entry| now.duration_since(entry.window_start) < window);
                    *last = now;
                }
            }

            let mut clients = clients.write().await;

            let entry = clients.entry(ip.clone()).or_insert(RateEntry {
                count: 0,
                window_start: now,
            });

            if now.duration_since(entry.window_start) > window {
                entry.count = 0;
                entry.window_start = now;
            }

            entry.count += 1;

            if entry.count > max_requests {
                drop(clients);
                let mut response = Response::new(axum::body::Body::from("Rate limit exceeded"));
                *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
                return Ok(response);
            }

            drop(clients);
            inner.call(req).await
        })
    }
}

// ---------------------------------------------------------------------------
// Path-based rate limit service
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct PathRateLimitService<S> {
    inner: S,
    max_requests: u32,
    window: Duration,
    paths: Vec<String>,
    proxy_config: ProxyConfig,
    clients: Arc<RwLock<HashMap<String, RateEntry>>>,
    last_cleanup: Arc<tokio::sync::RwLock<Instant>>,
}

impl<S, ReqBody> Service<Request<ReqBody>> for PathRateLimitService<S>
where
    S: Service<Request<ReqBody>, Response = Response> + Send + Clone + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let clients = self.clients.clone();
        let max_requests = self.max_requests;
        let window = self.window;
        let paths = self.paths.clone();
        let last_cleanup = self.last_cleanup.clone();
        let proxy_config = self.proxy_config.clone();

        let path = req.uri().path().to_string();

        Box::pin(async move {
            // Only apply rate limiting to configured paths
            if !paths.iter().any(|p| path.starts_with(p)) {
                return inner.call(req).await;
            }

            let ip = extract_client_ip(&req, &proxy_config);

            let now = Instant::now();

            // Periodic cleanup every 5 minutes
            {
                let mut last = last_cleanup.write().await;
                if now.duration_since(*last) > Duration::from_secs(300) {
                    let mut clients_guard = clients.write().await;
                    clients_guard
                        .retain(|_, entry| now.duration_since(entry.window_start) < window);
                    *last = now;
                }
            }

            let mut clients = clients.write().await;

            let key = format!("{}:{}", ip, path);
            let entry = clients.entry(key).or_insert(RateEntry {
                count: 0,
                window_start: now,
            });

            if now.duration_since(entry.window_start) > window {
                entry.count = 0;
                entry.window_start = now;
            }

            entry.count += 1;

            if entry.count > max_requests {
                drop(clients);
                let mut response = Response::new(axum::body::Body::from("Rate limit exceeded"));
                *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
                return Ok(response);
            }

            drop(clients);
            inner.call(req).await
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{HeaderValue, Request};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn peer_addr(ip: &str) -> ConnectInfo<SocketAddr> {
        ConnectInfo(SocketAddr::new(IpAddr::from_str(ip).unwrap(), 12345))
    }

    fn fake_request_with_peer(ip: &str) -> Request<Body> {
        let mut req = Request::new(Body::empty());
        req.extensions_mut().insert(peer_addr(ip));
        req
    }

    fn fake_request_with_peer_and_xff(ip: &str, xff: &str) -> Request<Body> {
        let mut req = fake_request_with_peer(ip);
        req.headers_mut()
            .insert("x-forwarded-for", HeaderValue::from_str(xff).unwrap());
        req
    }

    // ---- TRUST_PROXY=false (default) ----

    #[test]
    fn direct_client_without_proxy_headers() {
        let config = ProxyConfig::default(); // trust_proxy = false
        let req = fake_request_with_peer("203.0.113.1");
        let ip = extract_client_ip(&req, &config);
        assert_eq!(ip, "203.0.113.1");
    }

    #[test]
    fn spoofed_xff_ignored_when_trust_proxy_false() {
        let config = ProxyConfig::default();
        let req = fake_request_with_peer_and_xff("203.0.113.1", "10.0.0.1, 192.168.1.1");
        let ip = extract_client_ip(&req, &config);
        // Must use direct peer, NOT the spoofed XFF
        assert_eq!(ip, "203.0.113.1");
    }

    // ---- TRUST_PROXY=true with trusted peer ----

    #[test]
    fn trusted_proxy_with_valid_xff() {
        let config = ProxyConfig {
            trust_proxy: true,
            trusted_proxy_ips: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
        };
        let req = fake_request_with_peer_and_xff("10.0.0.1", "203.0.113.1, 10.0.0.1");
        let ip = extract_client_ip(&req, &config);
        assert_eq!(ip, "203.0.113.1");
    }

    #[test]
    fn untrusted_peer_xff_ignored() {
        let config = ProxyConfig {
            trust_proxy: true,
            trusted_proxy_ips: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
        };
        // Peer is 203.0.113.1 which is NOT in trusted_proxy_ips
        let req = fake_request_with_peer_and_xff("203.0.113.1", "10.0.0.1");
        let ip = extract_client_ip(&req, &config);
        // Must use direct peer
        assert_eq!(ip, "203.0.113.1");
    }

    // ---- Multiple proxy hops ----

    #[test]
    fn multiple_proxy_hops() {
        let config = ProxyConfig {
            trust_proxy: true,
            trusted_proxy_ips: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
        };
        // Client → proxy1 → proxy2 (our direct peer is proxy1)
        let req = fake_request_with_peer_and_xff("10.0.0.1", "198.51.100.1, 10.0.0.2, 10.0.0.1");
        let ip = extract_client_ip(&req, &config);
        // Leftmost entry is the original client
        assert_eq!(ip, "198.51.100.1");
    }

    // ---- Malformed forwarding header ----

    #[test]
    fn malformed_xff_falls_back_to_peer() {
        let config = ProxyConfig {
            trust_proxy: true,
            trusted_proxy_ips: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
        };
        let mut req = fake_request_with_peer("10.0.0.1");
        req.headers_mut()
            .insert("x-forwarded-for", HeaderValue::from_static("not-an-ip"));
        let ip = extract_client_ip(&req, &config);
        assert_eq!(ip, "10.0.0.1");
    }

    // ---- IPv4 and IPv6 ----

    #[test]
    fn ipv6_client_address() {
        let config = ProxyConfig::default();
        let mut req = Request::new(Body::empty());
        req.extensions_mut().insert(ConnectInfo(SocketAddr::new(
            IpAddr::from_str("2001:db8::1").unwrap(),
            12345,
        )));
        let ip = extract_client_ip(&req, &config);
        assert_eq!(ip, "2001:db8::1");
    }

    #[test]
    fn ipv6_xff_from_trusted_proxy() {
        let config = ProxyConfig {
            trust_proxy: true,
            trusted_proxy_ips: vec![IpAddr::from_str("10.0.0.1").unwrap()],
        };
        let mut req = fake_request_with_peer("10.0.0.1");
        req.headers_mut().insert(
            "x-forwarded-for",
            HeaderValue::from_static("2001:db8::42, 10.0.0.1"),
        );
        let ip = extract_client_ip(&req, &config);
        assert_eq!(ip, "2001:db8::42");
    }

    // ---- Trust all peers (empty list) ----

    #[test]
    fn trust_all_peers_when_list_empty() {
        let config = ProxyConfig {
            trust_proxy: true,
            trusted_proxy_ips: vec![],
        };
        let req = fake_request_with_peer_and_xff("172.16.0.1", "198.51.100.99");
        let ip = extract_client_ip(&req, &config);
        assert_eq!(ip, "198.51.100.99");
    }

    // ---- XFF with whitespace ----

    #[test]
    fn xff_with_whitespace() {
        let config = ProxyConfig {
            trust_proxy: true,
            trusted_proxy_ips: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
        };
        let req = fake_request_with_peer_and_xff("10.0.0.1", "  203.0.113.50 , 10.0.0.1");
        let ip = extract_client_ip(&req, &config);
        assert_eq!(ip, "203.0.113.50");
    }

    // ---- No ConnectInfo ----

    #[test]
    fn missing_peer_returns_unknown() {
        let config = ProxyConfig::default();
        let req = Request::new(Body::empty());
        let ip = extract_client_ip(&req, &config);
        assert_eq!(ip, "unknown");
    }

    // ---- From env defaults ----

    #[test]
    fn proxy_config_from_env_defaults() {
        // Without setting env vars, defaults to trust_proxy=false
        let config = ProxyConfig::from_env();
        assert!(!config.trust_proxy);
        assert!(config.trusted_proxy_ips.is_empty());
    }

    // ---- Empty XFF ----

    #[test]
    fn empty_xff_falls_back_to_peer() {
        let config = ProxyConfig {
            trust_proxy: true,
            trusted_proxy_ips: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
        };
        let mut req = fake_request_with_peer("10.0.0.1");
        req.headers_mut()
            .insert("x-forwarded-for", HeaderValue::from_static(""));
        let ip = extract_client_ip(&req, &config);
        assert_eq!(ip, "10.0.0.1");
    }
}
