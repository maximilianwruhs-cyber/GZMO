use anyhow::Result;
use redis::AsyncCommands;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

const REDIS_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const REDIS_RECONNECT_BACKOFF: Duration = Duration::from_secs(15);
const CCR_PREFIX: &str = "gzmo:ccr:";

#[derive(Clone)]
pub struct CcrStore {
    client: Option<redis::Client>,
    conn: Arc<Mutex<Option<redis::aio::ConnectionManager>>>,
    next_retry: Arc<Mutex<Instant>>,
    ttl_secs: u64,
    enabled: bool,
}

impl std::fmt::Debug for CcrStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CcrStore")
            .field("client", &self.client)
            .field("ttl_secs", &self.ttl_secs)
            .field("enabled", &self.enabled)
            .finish_non_exhaustive()
    }
}

impl CcrStore {
    pub fn new(redis_cfg: &crate::config::RedisConfig, compress_cfg: &crate::config::ContextCompressConfig) -> Self {
        if !redis_cfg.enabled {
            return Self {
                client: None,
                conn: Arc::new(Mutex::new(None)),
                next_retry: Arc::new(Mutex::new(Instant::now())),
                ttl_secs: compress_cfg.ccr_ttl_secs,
                enabled: false,
            };
        }

        match redis::Client::open(redis_cfg.url.as_str()) {
            Ok(client) => Self {
                client: Some(client),
                conn: Arc::new(Mutex::new(None)),
                next_retry: Arc::new(Mutex::new(Instant::now())),
                ttl_secs: compress_cfg.ccr_ttl_secs,
                enabled: true,
            },
            Err(e) => {
                error!("Failed to open Redis client for CcrStore: {e}");
                Self {
                    client: None,
                    conn: Arc::new(Mutex::new(None)),
                    next_retry: Arc::new(Mutex::new(Instant::now())),
                    ttl_secs: compress_cfg.ccr_ttl_secs,
                    enabled: false,
                }
            }
        }
    }

    pub fn mock() -> Self {
        Self {
            client: None,
            conn: Arc::new(Mutex::new(None)),
            next_retry: Arc::new(Mutex::new(Instant::now())),
            ttl_secs: 3600,
            enabled: false,
        }
    }

    async fn conn(&self) -> Result<redis::aio::ConnectionManager> {
        let Some(ref client) = self.client else {
            anyhow::bail!("Redis not configured for CCR");
        };

        if let Some(c) = self.conn.lock().await.as_ref() {
            return Ok(c.clone());
        }

        {
            let mut next = self.next_retry.lock().await;
            if Instant::now() < *next {
                anyhow::bail!("redis reconnect backing off");
            }
            *next = Instant::now() + REDIS_RECONNECT_BACKOFF;
        }

        match tokio::time::timeout(REDIS_CONNECT_TIMEOUT, client.get_connection_manager()).await {
            Ok(Ok(conn)) => {
                info!("CcrStore Redis backend connected");
                *self.conn.lock().await = Some(conn.clone());
                Ok(conn)
            }
            Ok(Err(e)) => {
                warn!("CcrStore Redis unreachable: {e}");
                anyhow::bail!("redis reconnect failed: {e}")
            }
            Err(_) => {
                warn!("CcrStore Redis connect timed out");
                anyhow::bail!("redis connect timed out")
            }
        }
    }

    async fn drop_conn(&self) {
        *self.conn.lock().await = None;
    }

    pub async fn store(&self, session_id: &str, content: &str) -> Result<String> {
        let digest = Sha256::digest(content.as_bytes());
        let hash = format!("{:x}", digest)[..16].to_string();

        if !self.enabled {
            return Ok(hash);
        }

        let key = format!("{}{}:{}", CCR_PREFIX, session_id, hash);

        match self.conn().await {
            Ok(mut conn) => {
                let res: redis::RedisResult<()> = conn.set_ex(&key, content, self.ttl_secs).await;
                if let Err(e) = res {
                    self.drop_conn().await;
                    debug!("CcrStore Redis SETEX failed: {e}");
                }
            }
            Err(e) => {
                debug!("CcrStore Redis store failed, fail-open: {e}");
            }
        }

        Ok(hash)
    }

    pub async fn retrieve(&self, session_id: &str, hash: &str) -> Result<Option<String>> {
        if !self.enabled {
            return Ok(None);
        }

        let key = format!("{}{}:{}", CCR_PREFIX, session_id, hash);

        match self.conn().await {
            Ok(mut conn) => {
                let res: redis::RedisResult<Option<String>> = conn.get(&key).await;
                match res {
                    Ok(opt) => Ok(opt),
                    Err(e) => {
                        self.drop_conn().await;
                        debug!("CcrStore Redis GET failed: {e}");
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                debug!("CcrStore Redis retrieve failed: {e}");
                Ok(None)
            }
        }
    }
}
