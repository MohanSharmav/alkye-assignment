use crate::error::ApiResult;
use redis::aio::ConnectionManager;
use uuid::Uuid;

pub struct Cache {
    redis: ConnectionManager,
}

impl Cache {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> ApiResult<Option<T>> {
        match redis::cmd("GET")
            .arg(key)
            .query_async::<_, Option<String>>(&mut self.redis.clone())
            .await
        {
            Ok(Some(value)) => {
                let deserialized = serde_json::from_str(&value)?;
                Ok(Some(deserialized))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("Redis GET error for key {}: {}", key, e);
                Ok(None)
            }
        }
    }

    pub async fn set<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: usize,
    ) -> ApiResult<()> {
        let serialized = serde_json::to_string(value)?;
        match redis::cmd("SETEX")
            .arg(key)
            .arg(ttl_secs)
            .arg(serialized)
            .query_async::<_, ()>(&mut self.redis.clone())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!("Redis SET error for key {}: {}", key, e);
                Ok(())
            }
        }
    }

    pub async fn delete(&self, key: &str) -> ApiResult<()> {
        match redis::cmd("DEL")
            .arg(key)
            .query_async::<_, ()>(&mut self.redis.clone())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!("Redis DELETE error for key {}: {}", key, e);
                Ok(())
            }
        }
    }
}

pub fn cache_key_for_user_tasks(user_id: Uuid) -> String {
    format!("user_tasks:{}", user_id)
}
