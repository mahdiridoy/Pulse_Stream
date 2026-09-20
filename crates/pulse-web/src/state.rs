use crate::cache::RedisCache;
use crate::db::repository::Repository;
use pulse_core::service::MovieBoxService;
use std::sync::Arc;

pub struct AppState {
    pub service: Arc<MovieBoxService>,
    pub repo: Repository,
    pub jwt_secret: String,
    pub redis: RedisCache,
}

impl AppState {
    pub fn new(
        service: Arc<MovieBoxService>,
        repo: Repository,
        jwt_secret: String,
        redis: RedisCache,
    ) -> Self {
        Self {
            service,
            repo,
            jwt_secret,
            redis,
        }
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            service: Arc::clone(&self.service),
            repo: Repository::new(self.repo.pool().clone()),
            jwt_secret: self.jwt_secret.clone(),
            redis: self.redis.clone(),
        }
    }
}
