use serde::Deserialize;
use std::fs::{File, metadata};
use std::io::Read;
use once_cell::sync::Lazy;
use serde_yaml;

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub enable_conversation_log: bool,
    pub conversation_log_path: String, // 例如 /var/log/chatig/conversation.log
}

// ---------------------------------------------- Config ----------------------------------------------
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub temp_docs_path: String,
    pub port: u16,
    pub metrics_port: u16,
    pub ipv6_enabled: bool,
    pub ipv6_port:u16,
    pub https_enabled: bool,
    pub request_timeout: u64,
    pub connection_timeout: u64,
    pub request_retry: u32,
    pub model_health_check_interval: u64,
    pub database: String,
    pub connection_num: u32,
    pub database_type: String,
    pub rate_limit_tps: usize,
    pub rate_limit_bucket_capacity: usize,
    pub rate_limit_refill_interval: u64,
    pub rate_limit_enbled: bool,
    pub auth_local_enabled: bool,
    pub auth_remote_enabled: bool,
    pub auth_remote_server: String,
    pub coil_enabled: bool,
    pub metrics_enable: bool,
    pub cloud_region_id: String,
    pub cloud_region_name: String,
    pub server_cert_file: String,
    pub chain_cert_file: String,
    pub key_file:String,
    pub multi_ip: Vec<String>,
    pub connections_per_server: usize,
    pub auth_cache_time: u64,
    pub auth_cache_capacity: usize,
    pub mind_ie_metrics_update_interval: usize,
    pub logging: LoggingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            temp_docs_path: "/root/.chatig/data/temp_docs".to_string(),
            port: 8081,
            metrics_port: 8082,
            ipv6_enabled: false,
            ipv6_port: 8003,
            https_enabled: false,
            request_timeout: 300,
            connection_timeout: 10,
            request_retry: 3,
            model_health_check_interval:300,
            database: "postgres://chatig:chatig@localhost/chatig".to_string(),
            connection_num: 10,
            database_type: "pgsql".to_string(),
            rate_limit_tps: 1000,
            rate_limit_bucket_capacity: 2000,
            rate_limit_refill_interval: 100,
            rate_limit_enbled: false,
            auth_local_enabled: false,
            auth_remote_enabled: false,
            auth_remote_server: "".to_string(),
            coil_enabled: false,
            metrics_enable: false,
            cloud_region_id: "".to_string(),
            cloud_region_name: "".to_string(),
            server_cert_file: "/etc/chatig/https/server_cert_file.crt".to_string(),
            chain_cert_file: "/etc/chatig/https/chain_cert_file.crt".to_string(),
            key_file: "/etc/chatig/https/key_file.key".to_string(),
            multi_ip: vec![
                "".to_string(),
            ],
            connections_per_server: 32,
            auth_cache_time: 1200,
            auth_cache_capacity: 3000,
            mind_ie_metrics_update_interval: 1,
            logging: LoggingConfig::default(), 
        }
    }
}


impl Config {
    pub fn load_config() -> Config {
        let config_path = if metadata("/etc/chatig/configs.yaml").is_ok() {
            "/etc/chatig/configs.yaml"
        } else {
            "src/configs/configs.yaml"
        };
        let mut file = File::open(config_path).expect("Failed to open config file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read config file");
        serde_yaml::from_str(&contents).expect("Failed to parse config file")

    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enable_conversation_log: true,
            conversation_log_path: "/var/log/chatig/conversation.log".to_string(),
        }
    }
}

// 全局静态配置对象
pub static GLOBAL_CONFIG: Lazy<Config> = Lazy::new(|| Config::load_config());