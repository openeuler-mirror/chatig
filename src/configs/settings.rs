use serde::Deserialize;
use std::fs::{File, metadata};
use std::io::Read;
use serde_yaml;

// ---------------------------------------------- Server Config ----------------------------------------------
// ChatChat API
#[derive(Debug, Deserialize, Clone)]
pub struct ChatChat{
    pub kb_chat: String,
    pub upload_temp_docs: String,
    pub file_chat: String,
    pub completion: String,
    pub model_name: String,
}

// EulerCopilot API
#[derive(Debug, Deserialize, Clone)]
pub struct EulerCopilot{
    pub get_answer: String,
    pub get_stream_answer: String,
}

// Configuration file
#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub chatchat: ChatChat,
    pub euler_copilot: EulerCopilot,
}

pub fn load_server_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let config_path = if metadata("/etc/chatig/configs.yaml").is_ok() {
        "/etc/chatig/servers_configs.yaml"
    } else {
        "src/configs/servers_configs.yaml"
    };
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: ServerConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}

// ---------------------------------------------- Config ----------------------------------------------
// API Key
#[derive(Debug, Deserialize, Clone)]
pub struct ApiKey{
    pub value: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub temp_docs_path: String,
    pub apikey: ApiKey,
    pub port: u16,
    pub database: String,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = if metadata("/etc/chatig/configs.yaml").is_ok() {
        "/etc/chatig/configs.yaml"
    } else {
        "src/configs/configs.yaml"
    };
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_yaml::from_str(&contents)?;
    Ok(config)
}