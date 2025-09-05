use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;
use actix_web::web;
use serde_json::Value;

use crate::configs::settings::Config;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationLog {
    pub conversation_id: String,
    pub input: String,
    pub output: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_ms: i64,
    pub rounds: u32,
    pub history: Option<Value>,
}

#[derive(Clone)]
pub struct AppState {
    pub cfg: Config,
}


pub async fn append_conv_log(state: web::Data<AppState>, log: ConversationLog) -> std::io::Result<()> {
    eprintln!("DEBUG append_conv_log CALLED");
    if !state.cfg.logging.enable_conversation_log {
        eprintln!("DEBUG append_conv_log skipped: disabled in config");
        return Ok(());
    }
    let path = &state.cfg.logging.conversation_log_path;
    eprintln!("DEBUG append_conv_log path={}", path);
  
    if let Some(dir) = Path::new(path).parent() {
        if !dir.exists() {
            create_dir_all(dir)?;
        }
    }
    let line = serde_json::to_string(&log).unwrap();
    eprintln!("DEBUG about to spawn_blocking, line={}", line);
    let path_owned = path.clone();
    actix_web::rt::task::spawn_blocking(move || -> std::io::Result<()> {
        eprintln!("DEBUG spawn_blocking actually writing to {}", path_owned);
        let mut f = OpenOptions::new().create(true).append(true).open(&path_owned)?;
        writeln!(f, "{}", line)?;
        Ok(())
    }).await.unwrap()
}
