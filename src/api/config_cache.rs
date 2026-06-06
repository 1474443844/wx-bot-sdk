use rand::Rng;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use super::{WeixinApiOptions, get_config};
use crate::util::Logger;

#[derive(Clone, Debug, Default)]
pub struct CachedConfig {
    pub typing_ticket: Option<String>,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    config: CachedConfig,
    fetched_at: Instant,
    ttl: Duration,
    fail_count: u32,
}

const DEFAULT_TTL_MS: u64 = 24 * 60 * 60 * 1000;
const INITIAL_RETRY_MS: u64 = 2_000;
const MAX_RETRY_MS: u64 = 60 * 60 * 1000;

#[derive(Clone, Debug)]
pub struct WeixinConfigManager {
    cache: HashMap<String, CacheEntry>,
    api_opts: WeixinApiOptions,
    log: Logger,
}

impl WeixinConfigManager {
    pub fn new(api_opts: WeixinApiOptions, log: Logger) -> Self {
        Self {
            cache: HashMap::new(),
            api_opts,
            log,
        }
    }

    pub async fn get_for_user(
        &mut self,
        user_id: &str,
        context_token: Option<&str>,
    ) -> CachedConfig {
        let should_fetch = self
            .cache
            .get(user_id)
            .map(|e| e.fetched_at.elapsed() >= e.ttl)
            .unwrap_or(true);
        if should_fetch {
            if let Ok(resp) = get_config(&self.api_opts, user_id, context_token).await {
                if resp.ret == Some(0) {
                    let ttl = rand::thread_rng().gen_range(0..DEFAULT_TTL_MS.max(1));
                    let entry = CacheEntry {
                        config: CachedConfig {
                            typing_ticket: Some(resp.typing_ticket.unwrap_or_default()),
                        },
                        fetched_at: Instant::now(),
                        ttl: Duration::from_millis(ttl),
                        fail_count: 0,
                    };
                    self.cache.insert(user_id.to_string(), entry);
                    return self.cache[user_id].config.clone();
                }
            } else {
                self.log.warn(format!("getConfig failed for {user_id}"));
            }
            let prev = self.cache.get(user_id).map(|e| e.fail_count).unwrap_or(0);
            let retry =
                (INITIAL_RETRY_MS.saturating_mul(2_u64.saturating_pow(prev))).min(MAX_RETRY_MS);
            self.cache
                .entry(user_id.to_string())
                .and_modify(|e| {
                    e.fail_count = prev + 1;
                    e.ttl = Duration::from_millis(retry);
                    e.fetched_at = Instant::now();
                })
                .or_insert(CacheEntry {
                    config: CachedConfig {
                        typing_ticket: Some(String::new()),
                    },
                    fetched_at: Instant::now(),
                    ttl: Duration::from_millis(retry),
                    fail_count: 1,
                });
        }
        self.cache
            .get(user_id)
            .map(|e| e.config.clone())
            .unwrap_or_default()
    }

    pub fn clear(&mut self, user_id: &str) {
        self.cache.remove(user_id);
    }
    pub fn clear_all(&mut self) {
        self.cache.clear();
    }
}
