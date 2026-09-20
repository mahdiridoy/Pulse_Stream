use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use url::Url;

use hickory_resolver::TokioResolver;
use hickory_resolver::config::{
    CLOUDFLARE_IPS, GOOGLE_IPS, LookupIpStrategy, NameServerConfigGroup, QUAD9_IPS, ResolverConfig,
};
use hickory_resolver::name_server::TokioConnectionProvider;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum SsrfError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Blocked private IP address: {0}")]
    PrivateIp(String),
    #[error("Blocked loopback address: {0}")]
    Loopback(String),
    #[error("Blocked link-local address: {0}")]
    LinkLocal(String),
    #[error("Blocked unspecified address: {0}")]
    Unspecified(String),
    #[error("Blocked multicast/reserved address: {0}")]
    MulticastReserved(String),
    #[error("Blocked cloud metadata address: {0}")]
    Metadata(String),
    #[error("Blocked IPv6 loopback address")]
    IPv6Loopback,
    #[error("Blocked IPv6 link-local address: {0}")]
    IPv6LinkLocal(String),
    #[error("Blocked IPv6 unique-local address: {0}")]
    IPv6UniqueLocal(String),
    #[error("Blocked IPv4-mapped IPv6 address: {0}")]
    IPv4Mapped(String),
    #[error("Blocked mixed DNS results (any forbidden address present)")]
    MixedDnsBlocked,
    #[error("DNS resolution failed for {0}: {1}")]
    DnsResolutionFailed(String, String),
    #[error("No addresses resolved for {0}")]
    NoAddresses(String),
    #[error("Blocked file:// scheme")]
    FileScheme,
    #[error("Redirect to blocked address: {0}")]
    RedirectBlocked(String),
}

// ---------------------------------------------------------------------------
// IP validation (comprehensive)
// ---------------------------------------------------------------------------

/// Check if an IPv4 address is in a blocked range.
fn is_blocked_ipv4(ip: Ipv4Addr) -> Option<&'static str> {
    if ip.is_loopback() {
        return Some("loopback");
    }
    if ip.is_private() {
        return Some("private/RFC1918");
    }
    if ip.is_link_local() {
        return Some("link-local");
    }
    if ip.is_broadcast() {
        return Some("broadcast");
    }
    if ip.is_unspecified() {
        return Some("unspecified");
    }
    if ip.is_documentation() {
        return Some("documentation");
    }
    // Multicast: 224.0.0.0/4
    if ip.octets()[0] & 0xF0 == 224 {
        return Some("multicast");
    }
    // Reserved: 240.0.0.0/4 (except 255.255.255.255 which is broadcast)
    if ip.octets()[0] >= 240 && ip != Ipv4Addr::new(255, 255, 255, 255) {
        return Some("reserved");
    }
    // Cloud metadata
    if ip.octets() == [169, 254, 169, 254] {
        return Some("cloud-metadata");
    }
    None
}

/// Check if an IPv6 address is in a blocked range.
fn is_blocked_ipv6(ip: Ipv6Addr) -> Option<&'static str> {
    if ip.is_loopback() {
        return Some("loopback");
    }
    if ip.is_unspecified() {
        return Some("unspecified");
    }
    if ip.is_unicast_link_local() {
        return Some("link-local");
    }
    if ip.is_unique_local() {
        return Some("unique-local");
    }
    // Multicast: ff00::/8
    if ip.is_multicast() {
        return Some("multicast");
    }
    // Documentation: 2001:db8::/32
    if ip.segments()[0] == 0x2001 && ip.segments()[1] == 0x0db8 {
        return Some("documentation");
    }
    // IPv4-mapped IPv6: ::ffff:0:0/96
    if ip
        .to_ipv4_mapped()
        .is_some_and(|v4| is_blocked_ipv4(v4).is_some())
    {
        return Some("ipv4-mapped");
    }
    None
}

/// Validate a single IP address against SSRF policy.
/// Returns Ok(()) if safe, Err with reason if blocked.
pub fn validate_ip(ip: IpAddr) -> Result<(), SsrfError> {
    let ip_str = ip.to_string();
    match ip {
        IpAddr::V4(v4) => {
            if let Some(reason) = is_blocked_ipv4(v4) {
                return Err(match reason {
                    "loopback" => SsrfError::Loopback(ip_str),
                    "private/RFC1918" => SsrfError::PrivateIp(ip_str),
                    "link-local" => SsrfError::LinkLocal(ip_str),
                    "unspecified" => SsrfError::Unspecified(ip_str),
                    "broadcast" | "reserved" | "documentation" | "multicast" => {
                        SsrfError::MulticastReserved(ip_str)
                    }
                    "cloud-metadata" => SsrfError::Metadata(ip_str),
                    _ => SsrfError::PrivateIp(ip_str),
                });
            }
        }
        IpAddr::V6(v6) => {
            if let Some(reason) = is_blocked_ipv6(v6) {
                return Err(match reason {
                    "loopback" => SsrfError::IPv6Loopback,
                    "unspecified" => SsrfError::Unspecified(ip_str),
                    "link-local" => SsrfError::IPv6LinkLocal(ip_str),
                    "unique-local" => SsrfError::IPv6UniqueLocal(ip_str),
                    "multicast" | "documentation" => SsrfError::MulticastReserved(ip_str),
                    "ipv4-mapped" => SsrfError::IPv4Mapped(ip_str),
                    _ => SsrfError::PrivateIp(ip_str),
                });
            }
        }
    }
    Ok(())
}

/// Validate ALL resolved addresses. If ANY address is forbidden, reject the
/// entire set (DNS rebinding protection).
pub fn validate_all_ips(ips: &[IpAddr]) -> Result<(), SsrfError> {
    for ip in ips {
        validate_ip(*ip)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// URL validation (static, before DNS)
// ---------------------------------------------------------------------------

pub fn validate_url(url_str: &str) -> Result<Url, SsrfError> {
    let url = Url::parse(url_str).map_err(|e| SsrfError::InvalidUrl(e.to_string()))?;

    match url.scheme() {
        "http" | "https" => {}
        "file" => return Err(SsrfError::FileScheme),
        _ => {
            return Err(SsrfError::InvalidUrl(format!(
                "Unsupported scheme: {}",
                url.scheme()
            )));
        }
    }

    let host = url
        .host_str()
        .ok_or_else(|| SsrfError::InvalidUrl("No host".into()))?;

    // Strip IPv6 brackets for IP parsing: [::1] -> ::1
    let host_stripped = host
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(host);

    // If the host is a literal IP, validate it immediately
    if let Ok(ip) = host_stripped.parse::<IpAddr>() {
        validate_ip(ip)?;
    }

    // Block well-known dangerous hostnames (check both with and without brackets)
    let blocked_hosts = [
        "localhost",
        "127.0.0.1",
        "0.0.0.0",
        "::1",
        "[::1]",
        "metadata.google.internal",
    ];
    if blocked_hosts.iter().any(|&h| host.eq_ignore_ascii_case(h)) {
        return Err(SsrfError::Loopback(host.to_string()));
    }

    Ok(url)
}

// ---------------------------------------------------------------------------
// SSRF-aware DNS resolver (wraps hickory-resolver)
// ---------------------------------------------------------------------------

const FALLBACK_DNS_PORT: u16 = 53;

fn fallback_servers() -> Vec<std::net::IpAddr> {
    let mut servers = Vec::with_capacity(CLOUDFLARE_IPS.len() + GOOGLE_IPS.len() + QUAD9_IPS.len());
    servers.extend_from_slice(CLOUDFLARE_IPS);
    servers.extend_from_slice(GOOGLE_IPS);
    servers.extend_from_slice(QUAD9_IPS);
    servers
}

fn fallback_config() -> ResolverConfig {
    ResolverConfig::from_parts(
        None,
        Vec::new(),
        NameServerConfigGroup::from_ips_clear(&fallback_servers(), FALLBACK_DNS_PORT, true),
    )
}

fn build_base_resolver() -> TokioResolver {
    let mut builder = match TokioResolver::builder_tokio() {
        Ok(builder) => builder,
        Err(_) => TokioResolver::builder_with_config(
            fallback_config(),
            TokioConnectionProvider::default(),
        ),
    };
    builder.options_mut().ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
    builder.build()
}

/// DNS resolver that validates every resolved address against SSRF policy.
#[derive(Clone)]
pub struct SsrfValidatingResolver {
    inner: Arc<TokioResolver>,
}

impl SsrfValidatingResolver {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(build_base_resolver()),
        }
    }
}

impl Default for SsrfValidatingResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolve for SsrfValidatingResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = Arc::clone(&self.inner);
        let hostname = name.as_str().to_owned();
        Box::pin(async move {
            // Resolve both A and AAAA
            let lookup = resolver.lookup_ip(name.as_str()).await.map_err(
                |e| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(SsrfError::DnsResolutionFailed(
                        hostname.clone(),
                        e.to_string(),
                    ))
                },
            )?;

            let ips: Vec<IpAddr> = lookup.into_iter().collect();

            if ips.is_empty() {
                return Err(Box::new(SsrfError::NoAddresses(hostname))
                    as Box<dyn std::error::Error + Send + Sync>);
            }

            // Validate ALL resolved addresses
            validate_all_ips(&ips)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

            let addrs: Addrs = Box::new(ips.into_iter().map(|address| SocketAddr::new(address, 0)));
            Ok(addrs)
        })
    }
}

// ---------------------------------------------------------------------------
// SSRF-protected HTTP client builder
// ---------------------------------------------------------------------------

/// Build a reqwest::Client with SSRF-protected DNS resolution.
/// Every outbound connection will resolve DNS through the SSRF validator,
/// which blocks private, loopback, link-local, metadata, and other
/// reserved IP ranges.
pub fn ssrf_protected_client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .dns_resolver(Arc::new(SsrfValidatingResolver::new()))
        .redirect(reqwest::redirect::Policy::limited(10))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .tcp_nodelay(true)
        .tcp_keepalive(Some(Duration::from_secs(45)))
        .pool_idle_timeout(Some(Duration::from_secs(90)))
        .pool_max_idle_per_host(8)
}

/// Build a quick ssrf-protected client (for one-off use).
pub async fn ssrf_protected_client() -> reqwest::Client {
    ssrf_protected_client_builder().build().unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Static URL validation ---

    #[test]
    fn test_block_localhost() {
        assert!(validate_url("http://localhost/admin").is_err());
        assert!(validate_url("http://127.0.0.1/admin").is_err());
        assert!(validate_url("http://[::1]/admin").is_err());
    }

    #[test]
    fn test_block_private_ip() {
        assert!(validate_url("http://10.0.0.1/admin").is_err());
        assert!(validate_url("http://192.168.1.1/admin").is_err());
        assert!(validate_url("http://172.16.0.1/admin").is_err());
        assert!(validate_url("http://172.31.255.255/admin").is_err());
    }

    #[test]
    fn test_block_metadata() {
        assert!(validate_url("http://169.254.169.254/latest/meta-data").is_err());
    }

    #[test]
    fn test_block_file() {
        assert!(validate_url("file:///etc/passwd").is_err());
    }

    #[test]
    fn test_block_unsupported_scheme() {
        assert!(validate_url("ftp://example.com/file").is_err());
        assert!(validate_url("gopher://example.com").is_err());
        assert!(validate_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_allow_public() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://8.8.8.8").is_ok());
        assert!(validate_url("https://cdn.example.com/path?q=1").is_ok());
    }

    #[test]
    fn test_block_0_0_0_0() {
        assert!(validate_url("http://0.0.0.0/admin").is_err());
    }

    #[test]
    fn test_no_host() {
        assert!(validate_url("http://").is_err());
    }

    // --- IP validation ---

    #[test]
    fn test_validate_ip_public() {
        assert!(validate_ip("8.8.8.8".parse().unwrap()).is_ok());
        assert!(validate_ip("1.1.1.1".parse().unwrap()).is_ok());
        assert!(validate_ip("93.184.216.34".parse().unwrap()).is_ok());
    }

    #[test]
    fn test_validate_ip_loopback() {
        assert!(validate_ip("127.0.0.1".parse().unwrap()).is_err());
        assert!(validate_ip("127.0.0.2".parse().unwrap()).is_err());
        assert!(validate_ip("::1".parse().unwrap()).is_err());
    }

    #[test]
    fn test_validate_ip_private() {
        assert!(validate_ip("10.0.0.1".parse().unwrap()).is_err());
        assert!(validate_ip("192.168.1.1".parse().unwrap()).is_err());
        assert!(validate_ip("172.16.0.1".parse().unwrap()).is_err());
    }

    #[test]
    fn test_validate_ip_link_local() {
        assert!(validate_ip("169.254.1.1".parse().unwrap()).is_err());
        assert!(validate_ip("169.254.255.255".parse().unwrap()).is_err());
    }

    #[test]
    fn test_validate_ip_metadata() {
        assert!(validate_ip("169.254.169.254".parse().unwrap()).is_err());
    }

    #[test]
    fn test_validate_ip_unspecified() {
        assert!(validate_ip("0.0.0.0".parse().unwrap()).is_err());
        assert!(validate_ip("::".parse().unwrap()).is_err());
    }

    #[test]
    fn test_validate_ip_multicast() {
        assert!(validate_ip("224.0.0.1".parse().unwrap()).is_err());
        assert!(validate_ip("239.255.255.255".parse().unwrap()).is_err());
    }

    #[test]
    fn test_validate_ip_reserved() {
        assert!(validate_ip("240.0.0.1".parse().unwrap()).is_err());
    }

    #[test]
    fn test_validate_ipv6_loopback() {
        assert!(validate_ip("::1".parse().unwrap()).is_err());
    }

    #[test]
    fn test_validate_ipv6_link_local() {
        assert!(validate_ip("fe80::1".parse().unwrap()).is_err());
    }

    #[test]
    fn test_validate_ipv6_unique_local() {
        assert!(validate_ip("fc00::1".parse().unwrap()).is_err());
        assert!(validate_ip("fd00::1".parse().unwrap()).is_err());
    }

    #[test]
    fn test_validate_ipv6_multicast() {
        assert!(validate_ip("ff02::1".parse().unwrap()).is_err());
    }

    #[test]
    fn test_validate_ipv4_mapped_ipv6_private() {
        // ::ffff:10.0.0.1 maps to 10.0.0.1 (private)
        let ip: IpAddr = "::ffff:10.0.0.1".parse().unwrap();
        assert!(validate_ip(ip).is_err());
    }

    #[test]
    fn test_validate_ipv4_mapped_ipv6_public() {
        // ::ffff:8.8.8.8 maps to 8.8.8.8 (public)
        let ip: IpAddr = "::ffff:8.8.8.8".parse().unwrap();
        assert!(validate_ip(ip).is_ok());
    }

    #[test]
    fn test_validate_all_ips_rejects_mixed() {
        let ips = vec![
            "8.8.8.8".parse().unwrap(),
            "192.168.1.1".parse().unwrap(), // forbidden!
        ];
        assert!(validate_all_ips(&ips).is_err());
    }

    #[test]
    fn test_validate_all_ips_allows_clean() {
        let ips = vec!["8.8.8.8".parse().unwrap(), "1.1.1.1".parse().unwrap()];
        assert!(validate_all_ips(&ips).is_ok());
    }

    // --- Builder smoke test ---

    #[test]
    fn ssrf_protected_builder_creates_client() {
        let client = ssrf_protected_client_builder().build();
        assert!(client.is_ok());
    }

    #[test]
    fn fallback_config_contains_public_resolvers() {
        let config = fallback_config();
        let servers = config.name_servers();
        assert!(servers.len() >= 12);
        assert!(servers.iter().any(|server| server.socket_addr.ip()
            == std::net::IpAddr::V4(std::net::Ipv4Addr::new(1, 1, 1, 1))));
    }
}
