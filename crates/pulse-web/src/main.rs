use axum::{
    Router,
    response::Json,
    routing::{delete, get, post},
};
use serde_json::{Value, json};
use std::sync::Arc;
use tower_http::cors::AllowOrigin;
use tower_http::trace::TraceLayer;

pub mod auth;
pub mod cache;
pub mod db;
pub mod handlers;
pub mod middleware;
pub mod state;

pub use state::AppState;

use crate::cache::RedisCache;
use crate::middleware::rate_limit::{PathRateLimitLayer, ProxyConfig, RateLimitLayer};
use crate::middleware::security::SecurityHeadersLayer;
use crate::middleware::ssrf::ssrf_protected_client;
use std::time::Duration;

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "pulse-web",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://pulse:pulse_secret@localhost:5432/pulse_stream".into());

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::error!("JWT_SECRET environment variable is required but not set");
        std::process::exit(1);
    });

    let pool = match db::create_pool(&database_url).await {
        Ok(pool) => {
            tracing::info!("Connected to PostgreSQL");
            if let Err(e) = db::run_migrations(&pool).await {
                tracing::error!("Migration failed: {}", e);
            }
            pool
        }
        Err(e) => {
            tracing::error!(
                "Failed to connect to PostgreSQL: {}. Running without database.",
                e
            );
            panic!("Database connection required. Set DATABASE_URL environment variable.");
        }
    };

    let redis = RedisCache::new().await;

    let repo = db::repository::Repository::new(pool);
    // Use SSRF-protected client for addon operations (user-controlled URLs)
    let addon_client = ssrf_protected_client().await;
    let service =
        Arc::new(pulse_core::service::MovieBoxService::with_addon_http_client(addon_client));
    let state = AppState::new(service, repo, jwt_secret, redis);

    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".into());

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(
            move |origin: &http::HeaderValue, _| {
                let frontend_url_str = frontend_url.clone();
                origin
                    .to_str()
                    .map(|s| s == frontend_url_str || s == "http://localhost:3000")
                    .unwrap_or(false)
            },
        ))
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::DELETE,
            http::Method::PATCH,
        ])
        .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
        .allow_credentials(true);

    let rate_limit =
        RateLimitLayer::with_proxy_config(100, Duration::from_secs(60), ProxyConfig::from_env());
    let auth_rate_limit = PathRateLimitLayer::with_proxy_config(
        10,
        Duration::from_secs(60),
        vec!["/api/v1/auth".to_string()],
        ProxyConfig::from_env(),
    );

    let auth_routes = Router::new()
        .route("/register", post(handlers::v2::auth_handlers::register))
        .route("/login", post(handlers::v2::auth_handlers::login))
        .route("/logout", post(handlers::v2::auth_handlers::logout))
        .route("/me", get(handlers::v2::auth_handlers::me))
        .layer(auth_rate_limit);

    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/search", get(handlers::search::search))
        .route("/home", get(handlers::home::home))
        .route("/movies", get(handlers::movies::list_movies))
        .route("/movies/{id}", get(handlers::movies::get_movie))
        .route("/series", get(handlers::series::list_series))
        .route("/series/{id}", get(handlers::series::get_series))
        .route("/anime", get(handlers::anime::list_anime))
        .route(
            "/details/{provider}/{id}",
            get(handlers::details::get_details),
        )
        .route(
            "/streams/{provider}/{id}",
            get(handlers::streams::get_streams),
        )
        .route(
            "/subtitles/{provider}/{id}",
            get(handlers::subtitles::get_subtitles),
        )
        .route("/providers", get(handlers::providers::list_providers))
        .route("/tv/channels", get(handlers::tv::list_channels))
        .route("/iptv/channels", get(handlers::iptv::list_channels))
        .route(
            "/iptv/channels/{id}/stream",
            get(handlers::iptv::get_channel_stream),
        )
        .nest("/auth", auth_routes)
        .route(
            "/favorites",
            get(handlers::v2::favorites::list_favorites)
                .post(handlers::v2::favorites::add_favorite),
        )
        .route(
            "/favorites/{id}",
            delete(handlers::v2::favorites::remove_favorite),
        )
        .route(
            "/history",
            get(handlers::v2::history::list_history).post(handlers::v2::history::update_progress),
        )
        .route(
            "/history/{id}",
            delete(handlers::v2::history::remove_history),
        )
        .route(
            "/continue-watching",
            get(handlers::v2::history::continue_watching),
        )
        .route("/account", get(handlers::v2::account::get_account))
        .route(
            "/playlists",
            get(handlers::v2::playlists::list_playlists)
                .post(handlers::v2::playlists::add_playlist),
        )
        .route(
            "/playlists/{id}",
            delete(handlers::v2::playlists::remove_playlist)
                .patch(handlers::v2::playlists::toggle_playlist),
        )
        .route(
            "/addons",
            get(handlers::v2::addons::list_addons).post(handlers::v2::addons::add_addon),
        )
        .route(
            "/addons/{id}",
            delete(handlers::v2::addons::remove_addon).patch(handlers::v2::addons::toggle_addon),
        )
        .route("/admin/stats", get(handlers::admin::admin_stats))
        .route("/admin/users", get(handlers::admin::admin_list_users))
        .route(
            "/admin/users/{id}",
            delete(handlers::admin::admin_delete_user),
        );

    let app = Router::new()
        .nest("/api/v1", api_routes)
        .layer(SecurityHeadersLayer)
        .layer(rate_limit)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("{}:{}", host, port);

    tracing::info!("Pulse Stream server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
