use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use crate::configs::settings::GLOBAL_CONFIG;
use std::{error, fs};
use tokio_postgres::{Client, Error};
use crate::meta::models::traits::Model;
use chrono::Utc;

pub async fn setup_database() -> Result<Pool<PostgresConnectionManager<NoTls>>, Box<dyn error::Error>> {
    // Get a connection pool
    let pool = match get_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("获取连接池失败: {:?}", e);
            std::process::exit(1);
        }
    };
    let pool_clone = pool.clone();
    let mut client = match pool_clone.get().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("获取数据库连接失败: {:?}", e);
            std::process::exit(1);
        }
    };
    // Initialize the database (note that we pass a client from the pool for initialization)
    create_file_object_table(&client).await?;
    create_project_object_table(&client).await?;
    create_user_object_table(&client).await?;
    create_models_table(&mut client).await?;
    create_services_table(&mut client).await?;
    create_model_limits_table(&client).await?;
    create_user_key_table(&client).await?;
    create_user_key_models_table(&client).await?;
    create_user_model_limits_table(&client).await?;
    create_service_detail_table(&client).await?;

    Ok(pool) 
}

pub async fn get_pool() -> Result<Pool<PostgresConnectionManager<NoTls>>, Box<dyn error::Error>> {
    // Read the database URL from the environment variables if it exists, otherwise use the provided database_url
    let config = &*GLOBAL_CONFIG;
    println!("Using database URL: {}", config.database);

    // Create a Postgres connection manager
    let manager = PostgresConnectionManager::new_from_stringlike(config.database.clone(), NoTls)?;

    // Create a connection pool
    let pool = Pool::builder().build(manager).await?;

    Ok(pool)
}

// Create the file_object table
async fn create_file_object_table(client: &Client) -> Result<(), Error> {
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS file_object (
            id SERIAL PRIMARY KEY,
            object TEXT NOT NULL,
            bytes INTEGER NOT NULL,
            created_at BIGINT NOT NULL,
            filename TEXT NOT NULL,
            purpose TEXT NOT NULL
        );
    "#;

    client.execute(create_table_query, &[]).await?;

    Ok(())
}


// Create the project object table
async fn create_project_object_table(client: &Client) -> Result<(), Error> {
    // 创建表
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS project_object (
            id TEXT PRIMARY KEY,
            object TEXT NOT NULL,
            name TEXT NOT NULL,
            created_at BIGINT NOT NULL,
            archived_at BIGINT,
            status TEXT NOT NULL
        );
    "#;
    client.execute(create_table_query, &[]).await?;

    Ok(())
}

// Create the user object table
async fn create_user_object_table(client: &Client) -> Result<(), Error> {
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS user_object (
            id TEXT PRIMARY KEY,
            object TEXT NOT NULL,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            role TEXT NOT NULL,
            added_at BIGINT NOT NULL
        );
    "#;
    client.execute(create_table_query, &[]).await?;
    Ok(())
}

// Create the models table
async fn create_models_table(client: &mut Client) -> Result<(), Error> {
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS models (
        id TEXT PRIMARY KEY,
        object TEXT NOT NULL,
        model_name TEXT NOT NULL,
        request_url TEXT NOT NULL,
        created BIGINT NOT NULL,
        owned_by TEXT NOT NULL
    );
    "#;
    client.execute(create_table_query, &[]).await?;

    // Initialize the models table
    init_models_table(client).await?;

    Ok(())
}

async fn init_models_table(client: &mut Client) -> Result<(), Error> {
    println!("Initializing models table");
    let now = Utc::now();
    let timestamp = now.timestamp();
    let default_models = vec![
        Model {
            id: "Qwen2.5-14B-Instruct".to_string(),
            object: "model".to_string(),
            model_name: "qwen2.5-instruct".to_string(),
            request_url: "http://x.x.x.x:30007/v1/chat/completions".to_string(),
            created: timestamp,
            owned_by: "system".to_string(),
        },
        Model {
            id: "Qwen2.5-7B-Instruct".to_string(),
            object: "model".to_string(),
            model_name: "Qwen/Qwen2.5-7B-Instruct".to_string(),
            request_url: "http://x.x.x.x:8000/v1/chat/completions".to_string(),
            created: timestamp,
            owned_by: "system".to_string(),
        },
        Model {
            id: "Qwen3-0.6B".to_string(),
            object: "model".to_string(),
            model_name: "Qwen3-0.6B".to_string(), // 若未设置 served-model-name，可填 "/home/aisp/project/models/Qwen3-0.6B"
            request_url: "http://127.0.0.1:8000/v1/chat/completions".to_string(),
            created: timestamp,
            owned_by: "system".to_string(),
        },
        Model {
            id: "svc_bge_embedding".to_string(),
            object: "model".to_string(),
            model_name: "BAAI/bge-base-en-v1.5".to_string(), 
            request_url: "https://aigw-nmhhht.cucloud.cn/v1".to_string(),
            created: timestamp,
            owned_by: "system".to_string(),
        },

    ]; 

    let models_path = "/etc/chatig/models.yaml";
    let models: Vec<Model> = match fs::read_to_string(models_path) {
        Ok(content) => {
            serde_yaml::from_str(&content).unwrap_or_else(|_| {
                println!("Read YAML file successfully, but failed to parse it, using default data.");
                default_models.clone()
            })
        }
        Err(_) => {
            println!("Failed to read YAML file, using default data.");
            default_models.clone()
        }
    };

    let tx = client.transaction().await.unwrap();
    for model in &models {
        let _ = tx.execute(
            "INSERT INTO models (id, object, model_name, request_url, created, owned_by) VALUES ($1, $2, $3, $4, $5, $6);",
            &[
                &model.id,
                &model.object,
                &model.model_name,
                &model.request_url,
                &model.created,
                &model.owned_by,
            ],
        )
        .await;
    }
    tx.commit().await.unwrap();

    Ok(())
}

// Create the service table
pub async fn create_services_table(client: &Client) -> Result<(), Error> {
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS services (
            id TEXT PRIMARY KEY,
            servicetype TEXT NOT NULL,
            status TEXT NOT NULL,
            url TEXT NOT NULL,
            model_name TEXT NOT NULL,
            active_model TEXT NOT NULL,
            api_key TEXT NOT NULL,
            context_length FLOAT4 NOT NULL,
            tags TEXT NULL
        );
    "#;

    client.execute(create_table_query, &[]).await?;
    Ok(())
}

// Create the limits for models of each user
async fn create_model_limits_table(client: &Client) -> Result<(), Error> {
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS model_limits (
            id SERIAL PRIMARY KEY,
            model_name TEXT NOT NULL,
            max_requests TEXT NOT NULL,
            max_tokens TEXT NOT NULL,
            user_type TEXT NOT NULL,
            user_level TEXT NOT NULL,
            aicp_id TEXT
        );
    "#;
    client.execute(create_table_query, &[]).await?;
    Ok(())
}

// Create the usrkey table
async fn create_user_key_table(client: &Client) -> Result<(), Error> {
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS UserKeys (
            userkey VARCHAR(255) PRIMARY KEY,
            key_type VARCHAR(50) NULL
        );
    "#;
    client.execute(create_table_query, &[]).await?;
    Ok(())
}

// Create the user key models table
async fn create_user_key_models_table(client: &Client) -> Result<(), Error> {
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS UserKeysModels (
            id SERIAL PRIMARY KEY,  
            userkey VARCHAR(255) NOT NULL,  
            model VARCHAR(255) NOT NULL,
            service_id VARCHAR(255) NOT NULL,
            service_name VARCHAR(255) NOT NULL,
            user_id VARCHAR(255) NOT NULL,
            user_name VARCHAR(255) NOT NULL,
            manage_key VARCHAR(255) NOT NULL
        );
    "#;
    client.execute(create_table_query, &[]).await?;
    Ok(())
}

// Create the user service limits for models
async fn create_user_model_limits_table(client: &Client) -> Result<(), Error> {
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS user_model_limits (
            id SERIAL PRIMARY KEY,
            user_id TEXT NOT NULL,
            service_id TEXT,
            model_name TEXT NOT NULL,
            max_requests TEXT NOT NULL,
            max_tokens TEXT NOT NULL,
            user_name TEXT NOT NULL,
            service_name TEXT,
            aicp_id TEXT,
            FOREIGN KEY (aicp_id) REFERENCES services(id) ON DELETE SET NULL
        );
    "#;
    client.execute(create_table_query, &[]).await?;
    Ok(())
}



pub async fn create_service_detail_table(client: &Client) -> Result<(), Error> {
    // Create the service_detail table
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS services_detail (
            service_id TEXT PRIMARY KEY NOT NULL, -- 关联到 services 表的服务实例 ID
            metrics_url VARCHAR(255) NULL, -- 用于服务指标的监控
            health_check_url VARCHAR(255) NULL, -- 用于检查模型是否健康运行的 URL
            FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE CASCADE -- 外键约束，关联到 services 表的 id 字段
        );
    "#;
    client.execute(create_table_query, &[]).await?;

    Ok(())
}