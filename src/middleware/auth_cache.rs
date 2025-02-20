use std::time::{Duration, Instant};
use std::num::NonZeroUsize;
use lru::LruCache;

use crate::configs::settings::GLOBAL_CONFIG;

pub struct AuthCache {
    pub cache_manage: LruCache<String, (String, Instant)>,  // 存储 api_key -> (is_valid, expire_time)
    pub cache_model: LruCache<String, (String, Instant)>,  // 存储 api_key+model_name -> (user_id, expire_time)
}

impl AuthCache {
    // 创建新的缓存实例
    pub fn new() -> Self {
        let config = &*GLOBAL_CONFIG;
        let capacity = NonZeroUsize::new(config.auth_cache_capacity).expect("Capacity must be non-zero");// 限制最大缓存大小
        AuthCache {
            cache_manage: LruCache::new(capacity),
            cache_model: LruCache::new(capacity)
        }
    }

    pub fn check_cache_manage(&mut self, key: &str) -> Option<()> {
        println!("Auth current cache_manage content: {:?}", self.cache_manage);
        if let Some((_, expire_time)) = self.cache_manage.get(key) {  // 解包元组
            if Instant::now() < *expire_time {
                return Some(());  // 缓存有效
            } else {
                self.cache_manage.pop(key);  // 清空失效缓存
            }
        }
        None
    }

    // 设置manage缓存
    pub fn set_cache_manage(&mut self, key: String, ttl: Duration) {
        let expire_time = Instant::now() + ttl;
        self.cache_manage.put(key.clone(), (key, expire_time));
    }  

    // 检查model缓存是否有效
    pub fn check_cache_model(&mut self, key: &str) -> Option<String> {
        println!("Auth current cache_model content: {:?}", self.cache_model);
        if let Some((user_id, expire_time)) = self.cache_model.get(key) {
            if Instant::now() < *expire_time {
                return Some(user_id.clone());
            } else {
                self.cache_model.pop(key);
            }
        }
        None
    }

    // 设置model缓存
    pub fn set_cache_model(&mut self, key: &str, user_id: String, ttl: Duration) {
        let expire_time = Instant::now() + ttl;
        self.cache_model.put(key.to_string(), (user_id, expire_time));
    }
}
