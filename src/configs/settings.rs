use serde::Deserialize;
use std::fs::{File, metadata};
use std::io::Read;
use once_cell::sync::Lazy;
use serde_yaml;

// ---------------------------------------------- Server Config ----------------------------------------------
// ChatChat API
#[derive(Debug, Deserialize, Clone)]
pub struct ChatChat{
    pub kb_chat: String,
    pub upload_temp_docs: String,
    pub file_chat: String,
    pub model_name: String,
}

// EulerCopilot API
#[derive(Debug, Deserialize, Clone)]
pub struct EulerCopilot{
    pub get_answer: String,
    pub get_stream_answer: String,
}


// mindie API
#[derive(Debug, Deserialize, Clone)]
pub struct Embeddings{
    pub get_embedding: String,
    #[allow(dead_code)]
    pub model_name: String,
}


// Configuration file
#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub chatchat: ChatChat,
    pub euler_copilot: EulerCopilot,
    pub embeddings: Embeddings,
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

// ---------------------------------------------- Models Config ----------------------------------------------
#[derive(Deserialize, Debug, Clone)]
pub struct ModelConfig {
    pub qwen_models: QwenModels,
    pub glm_models: GLMModels,
    pub llama_models: LlamaModels,
}

#[derive(Deserialize, Debug, Clone)]
pub struct QwenModels {
    pub qwen2_5_7b_instruct: Qwen2_5_7BInstruct,
    pub qwen2_5_14b_instruct: Qwen2_5_14BInstruct,
    pub qwen_7b_chat: Qwen7BChat,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Qwen2_5_7BInstruct {
    pub model_name: String,
    pub max_tokens: u32,
    pub reqwest_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Qwen2_5_14BInstruct {
    pub model_name: String,
    pub max_tokens: u32,
    pub reqwest_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Qwen7BChat {
    pub model_name: String,
    pub max_tokens: u32,
    pub reqwest_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GLMModels {
    pub glm_7b_chat: GLM7BChat,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GLM7BChat {
    pub model_name: String,
    pub max_tokens: u32,
    pub reqwest_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LlamaModels {
    pub llama3_1_8b_instruct: Llama3_1_8BInstruct,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Llama3_1_8BInstruct {
    pub model_name: String,
    pub max_tokens: u32,
    pub reqwest_url: String,
}


pub fn load_models_config() -> Result<ModelConfig, Box<dyn std::error::Error>> {
    let config_path = if metadata("/etc/chatig/models_configs.yaml").is_ok() {
        "/etc/chatig/models_configs.yaml"
    } else {
        "src/configs/models_configs.yaml"
    };
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: ModelConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}

// ---------------------------------------------- Config ----------------------------------------------
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub temp_docs_path: String,
    pub port: u16,
    pub database: String,
    pub connection_num: u32,
    pub database_type: String,
}

impl Config {
    pub fn load_config() -> Config {
        let config_path = if metadata("/etc/chatig/configs.yaml").is_ok() {
            "/etc/chatig/configs.yaml"
        } else {
            "src/configs/configs.yaml"
        };
        let config_path = "src/configs/configs.yaml";
        let mut file = File::open(config_path).expect("Failed to open config file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read config file");
        serde_yaml::from_str(&contents).expect("Failed to parse config file")
    }
}

// 全局静态配置对象
pub static GLOBAL_CONFIG: Lazy<Config> = Lazy::new(|| Config::load_config());