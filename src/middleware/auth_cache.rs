use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct AuthCache {
    pub cache_manage: HashMap<String, (String, Instant)>,  // 存储 api_key -> (is_valid, expire_time)
    pub cache_model: HashMap<String, (String, Instant)>,  // 存储 api_key+app_key+model_name -> (user_id, expire_time)
}

impl AuthCache {
    // 创建新的缓存实例
    pub fn new() -> Self {
        AuthCache {
            cache_manage: HashMap::new(),
            cache_model: HashMap::new(),
        }
    }

    // 检查manage缓存是否有效
    pub fn check_cache_manage(&self, key: &str) -> Option<()> {
        if let Some((_, expire_time)) = self.cache_manage.get(key) {  // 解包元组
            if Instant::now() < *expire_time {
                // println!("Cache hit for key: {}", key);
                return Some(());  // 缓存有效
            }
        }
        None
    }

    // 设置manage缓存
    pub fn set_cache_manage(&mut self, key: String, ttl: Duration) {
        let expire_time = Instant::now() + ttl;
        self.cache_manage.insert(key.clone(), (key, expire_time));
    } 

    // 检查model缓存是否有效
    pub fn check_cache_model(&self, key: &str) -> Option<String> {
        if let Some((user_id, expire_time)) = self.cache_model.get(key) {
            if Instant::now() < *expire_time {
                // println!("Cache hit for key");
                return Some(user_id.clone());
            }
        }
        None
    }

    // 设置model缓存
    pub fn set_cache_model(&mut self, key: &str, user_id: String, ttl: Duration) {
        let expire_time = Instant::now() + ttl;
        self.cache_model.insert(key.to_string(), (user_id, expire_time));
    }
}
