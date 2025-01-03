use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use crate::configs::settings::GLOBAL_CONFIG;
use std::error;
use tokio_postgres::{Client, Error};

pub async fn setup_database() -> Result<Pool<PostgresConnectionManager<NoTls>>, Box<dyn error::Error>> {
    dotenv::dotenv().ok();

    // Read the database URL from the environment variables if it exists, otherwise use the provided database_url
    let config = &*GLOBAL_CONFIG;
    println!("Using database URL: {}", config.database);

    // Create a Postgres connection manager
    let manager = PostgresConnectionManager::new_from_stringlike(config.database.clone(), NoTls)?;

    // Create a connection pool
    let pool = Pool::builder().build(manager).await?;

    // Initialize the database (note that we pass a client from the pool for initialization)
    let pool_clone = pool.clone();
    let client: PooledConnection<'_, PostgresConnectionManager<NoTls>> = pool_clone.get().await?;
    create_file_object_table(&client).await?;
    create_invitation_code_table(&client).await?;
    create_project_object_table(&client).await?;
    create_user_object_table(&client).await?;

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