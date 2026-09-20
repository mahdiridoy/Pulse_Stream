use fred::clients::Client;
use fred::interfaces::{ClientLike, KeysInterface};
use fred::types::{Expiration, Value as RedisValue, config::Config};

pub struct RedisCache {
    pub client: Option<Client>,
}

impl RedisCache {
    pub async fn new() -> Self {
        let redis_url = match std::env::var("REDIS_URL") {
            Ok(url) => url,
            Err(_) => {
                tracing::warn!("REDIS_URL not set, running without Redis cache");
                return Self { client: None };
            }
        };

        let config = match Config::from_url(&redis_url) {
            Ok(config) => config,
            Err(e) => {
                tracing::warn!("Invalid Redis URL: {}. Running without cache.", e);
                return Self { client: None };
            }
        };

        let client = Client::new(config, None, None, None);

        match client.init().await {
            Ok(_) => {
                tracing::info!("Connected to Redis");
                Self {
                    client: Some(client),
                }
            }
            Err(e) => {
                tracing::warn!("Failed to connect to Redis: {}. Running without cache.", e);
                Self { client: None }
            }
        }
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let client = self.client.as_ref()?;
        let value: RedisValue = client.get(key).await.ok()?;
        let string_val: String = value.convert().ok()?;
        serde_json::from_str(&string_val).ok()
    }

    pub async fn set<T: serde::Serialize>(&self, key: &str, value: &T, ttl_secs: i64) {
        if let Some(client) = &self.client {
            if let Ok(json_val) = serde_json::to_string(value) {
                let _: Result<(), _> = client
                    .set(key, json_val, Some(Expiration::EX(ttl_secs)), None, false)
                    .await;
            }
        }
    }

    pub async fn delete(&self, key: &str) {
        if let Some(client) = &self.client {
            let _: Result<i64, _> = client.del(key).await;
        }
    }

    pub async fn health_check(&self) -> bool {
        match &self.client {
            Some(client) => {
                let _: Result<String, _> = client.ping(None).await;
                client.is_connected()
            }
            None => false,
        }
    }
}

impl Clone for RedisCache {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}
