use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct AuthCache {
    pub cache: HashMap<String, (String, Instant)>,  // 存储 api_key+model_name，返回 (user_id, expire_time)
}

impl AuthCache {
    // 创建新的缓存实例
    pub fn new() -> Self {
        AuthCache {
            cache: HashMap::new(),
        }
    }

    // 检查缓存是否有效
    pub fn check_cache(&self, key: &str) -> Option<()> {
        if let Some((_, expire_time)) = self.cache.get(key) {  // 解包元组
            if Instant::now() < *expire_time {
                // println!("Cache hit for key: {}", key);
                return Some(());  // 缓存有效
            }
        }
        None  // 缓存无效或不存在
    }

    // 设置缓存
    pub fn set_cache(&mut self, key: String, ttl: Duration) {
        let expire_time = Instant::now() + ttl;
        self.cache.insert(key.clone(), (key, expire_time));
    } 

    // // 检查缓存是否有效
    // pub fn check_cache(&self, key: &str) -> Option<String> {
    //     if let Some((user_id, expire_time)) = self.cache.get(key) {
    //         if Instant::now() < *expire_time {
    //             return Some(user_id.clone());
    //         }
    //     }
    //     None
    // }

    // // 设置缓存
    // pub fn set_cache(&mut self, key: &str, user_id: String, ttl: Duration) {
    //     let expire_time = Instant::now() + ttl;
    //     self.cache.insert(key.to_string(), (user_id, expire_time));
    // }
}
