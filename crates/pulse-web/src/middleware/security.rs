use axum::{http::HeaderValue, response::Response};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct SecurityHeadersLayer;

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeaders<S>;
    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeaders { inner }
    }
}

#[derive(Clone)]
pub struct SecurityHeaders<S> {
    inner: S,
}

impl<S, ReqBody> Service<axum::http::Request<ReqBody>> for SecurityHeaders<S>
where
    S: Service<axum::http::Request<ReqBody>, Response = Response> + Send + Clone + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        Box::pin(async move {
            let mut response = inner.call(req).await?;
            let headers = response.headers_mut();
            headers.insert(
                "x-content-type-options",
                HeaderValue::from_static("nosniff"),
            );
            headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
            headers.insert(
                "referrer-policy",
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );
            headers.insert(
                "permissions-policy",
                HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
            );
            // Content Security Policy
            headers.insert(
                "content-security-policy",
                HeaderValue::from_static(
                    "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"
                ),
            );
            // HSTS (only effective over HTTPS, but safe to include)
            headers.insert(
                "strict-transport-security",
                HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
            );
            Ok(response)
        })
    }
}
