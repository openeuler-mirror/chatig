use sqlx::{MySql, Postgres};
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::any::Any;
use std::fmt::{self, Debug};
use crate::configs::settings::GLOBAL_CONFIG;

pub(crate) static DB_MANAGER: OnceCell<Arc<RwLock<Box<dyn DbManager<Connection = Box<dyn Any + Send + Sync>>>>>> = OnceCell::new();

pub enum DbConnection {
    MySql(sqlx::pool::PoolConnection<sqlx::MySql>),
    Postgres(sqlx::pool::PoolConnection<sqlx::Postgres>),
}


#[async_trait]
pub trait DbManager: Send + Sync + Debug {
    type Connection: Send + Sync;

    async fn connection_pool(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn connect(&self) -> Result<Self::Connection, Box<dyn std::error::Error>>;
}

pub struct MySQL {
    pool: Option<sqlx::Pool<MySql>>, // 持有连接池
}

#[async_trait]
impl DbManager for MySQL {
    type Connection = Box<dyn Any + Send + Sync>;

    async fn connection_pool(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let config = &*GLOBAL_CONFIG;
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(config.connection_num)
            .connect(&config.database)
            .await?;
        self.pool.replace(pool); // 存储连接池
        Ok(())
    }

    async fn connect(&self) -> Result<Self::Connection, Box<dyn std::error::Error>> {
        if let Some(pool) = &self.pool {
            let conn = pool.acquire().await?;
            Ok(Box::new(conn))  // 将连接包裹成 Box<dyn Any>
        } else {
            Err("Connection pool is not initialized.".into())
        }
    }
}

impl Debug for MySQL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MySQL Database Manager")
    }
}

pub struct PgSQL {
    pool: Option<sqlx::Pool<Postgres>>,
}

#[async_trait]
impl DbManager for PgSQL {
    type Connection = Box<dyn Any + Send + Sync>;

    async fn connection_pool(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let config = &*GLOBAL_CONFIG;
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.connection_num)
            .connect(&config.database)
            .await?;
        self.pool.replace(pool);
        Ok(())
    }

    async fn connect(&self) -> Result<Self::Connection, Box<dyn std::error::Error>> {
        if let Some(pool) = &self.pool {
            let conn = pool.acquire().await?;
            Ok(Box::new(conn))  // 将连接包裹成 Box<dyn Any>
        } else {
            Err("Connection pool is not initialized.".into())
        }
    }
}

// 为 PgSQL 添加 Debug 实现
impl Debug for PgSQL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PostgreSQL Database Manager")
    }
}

pub async fn setup_database() -> Result<(), Box<dyn std::error::Error>> {
    let config = &*GLOBAL_CONFIG;

    let mut db_manager: Box<dyn DbManager<Connection = _>> = match &config.database_type as &str {
        "mysql" => Box::new(MySQL { pool: None }),
        "pgsql" => Box::new(PgSQL { pool: None }),
        _ => return Err("Unsupported database type".into()), // 如果不是 mysql 或 pgsql，返回错误
    };

    db_manager.connection_pool().await?; // 初始化连接池

    DB_MANAGER.set(Arc::new(RwLock::new(db_manager))).unwrap();
    Ok(())
}

pub async fn get_db_connection() -> Result<DbConnection, Box<dyn std::error::Error>> {
    let config = &*GLOBAL_CONFIG;
    let db_manager = DB_MANAGER.get().ok_or("DB_MANAGER is not initialized")?;
    let conn = db_manager.read().await.connect().await?;
    match config.database_type.as_str() {
        "mysql" => {
            let mysql_conn = conn
                .downcast::<sqlx::pool::PoolConnection<sqlx::MySql>>()
                .map_err(|_| "Failed to downcast to PoolConnection<MySql>")?;
            Ok(DbConnection::MySql(*mysql_conn))
        }
        "pgsql" => {
            let pg_conn = conn
                .downcast::<sqlx::pool::PoolConnection<sqlx::Postgres>>()
                .map_err(|_| "Failed to downcast to PoolConnection<Postgres>")?;
            Ok(DbConnection::Postgres(*pg_conn))
        }
        _ => Err("Unsupported database type".into()),
    }
}