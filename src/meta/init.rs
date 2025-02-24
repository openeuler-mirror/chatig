use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use crate::configs::settings::GLOBAL_CONFIG;
use std::{error, fs};
use tokio_postgres::{Client, Error};
use crate::meta::models::Model;
use chrono::Utc;

pub async fn setup_database() -> Result<Pool<PostgresConnectionManager<NoTls>>, Box<dyn error::Error>> {
    // Get a connection pool
    let pool = get_pool().await?;
    let pool_clone = pool.clone();
    let mut client: PooledConnection<'_, PostgresConnectionManager<NoTls>> = pool_clone.get().await?;

    // Initialize the database (note that we pass a client from the pool for initialization)
    create_file_object_table(&client).await?;
    create_invitation_code_table(&client).await?;
    create_project_object_table(&client).await?;
    create_user_object_table(&client).await?;
    create_models_table(&mut client).await?;
    create_services_table(&mut client).await?;
    create_models_service_table(&mut client).await?;
    create_model_limits_table(&client).await?;
    create_user_key_table(&client).await?;
    create_user_key_models_table(&client).await?;

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

// Create the invitation_code table
async fn create_invitation_code_table(client: &Client) -> Result<(), Error> {
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS invitation_code (
            id SERIAL PRIMARY KEY,
            users TEXT NOT NULL,
            origination TEXT,
            telephone TEXT,
            email TEXT,
            created_at BIGINT NOT NULL,
            code TEXT NOT NULL,
            UNIQUE (code)
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
            active_model TEXT NOT NULL
        );
    "#;

    client.execute(create_table_query, &[]).await?;
    Ok(())
}

// Create the models service table
pub async fn create_models_service_table(client: &Client) -> Result<(), Error> {
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS models_service (
            serviceid TEXT NOT NULL,
            modelid TEXT NOT NULL,
            PRIMARY KEY (serviceid, modelid),
            FOREIGN KEY (serviceid) REFERENCES services(id) ON DELETE CASCADE
        );
    "#;

    client.execute(create_table_query, &[]).await?;
    Ok(())
}

// Create the limits for models of each user
async fn create_model_limits_table(client: &Client) -> Result<(), Error> {
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS model_limits (
            model_name TEXT PRIMARY KEY,
            max_requests TEXT NOT NULL,
            max_tokens TEXT NOT NULL
        );
    "#;
    client.execute(create_table_query, &[]).await?;
    Ok(())
}

// Create the usrkey table
async fn create_user_key_table(client: &Client) -> Result<(), Error> {
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS UserKeys (
            userkey VARCHAR(255) PRIMARY KEY
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
            model VARCHAR(255) NOT NULL 
        );
    "#;
    client.execute(create_table_query, &[]).await?;
    Ok(())
}