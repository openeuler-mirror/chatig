use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use std::env;
use std::error;
use tokio_postgres::{Client, Error};

pub async fn setup_database(database_url: String) -> Result<Pool<PostgresConnectionManager<NoTls>>, Box<dyn error::Error>> {
    dotenv::dotenv().ok();

    // Read the database URL from the environment variables if it exists, otherwise use the provided database_url
    let env_database_url = env::var("DATABASE_URL").unwrap_or(database_url);
    println!("Using database URL: {}", env_database_url);

    // Create a Postgres connection manager
    let manager = PostgresConnectionManager::new_from_stringlike(env_database_url, NoTls)?;

    // Create a connection pool
    let pool = Pool::builder().build(manager).await?;

    // Initialize the database (note that we pass a client from the pool for initialization)
    let pool_clone = pool.clone();
    let client: PooledConnection<'_, PostgresConnectionManager<NoTls>> = pool_clone.get().await?;
    create_file_object_table(&client).await?;

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