use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value as JsonValue;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::{query, query_as, MySql, Postgres, Pool};
use sqlx::pool::PoolConnection;
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::any::Any;
use std::fmt::{self, Debug};
use std::error::Error;
use crate::configs::settings::GLOBAL_CONFIG;

pub(crate) static DB_MANAGER: OnceCell<Arc<RwLock<Box<dyn DbManager<Connection = Box<dyn Any + Send + Sync>>>>>> = OnceCell::new();

pub enum DbConnection {
    MySql(PoolConnection<MySql>),
    Postgres(PoolConnection<Postgres>),
}


#[async_trait]
pub trait DbManager: Send + Sync + Debug {
    type Connection: Send + Sync;

    async fn connection_pool(&mut self) -> Result<(), Box<dyn Error>>;
    async fn connect(&self) -> Result<Self::Connection, Box<dyn Error>>;
}

pub struct MySQL {
    pool: Option<Pool<MySql>>, // 持有连接池
}

#[async_trait]
impl DbManager for MySQL {
    type Connection = Box<dyn Any + Send + Sync>;

    async fn connection_pool(&mut self) -> Result<(), Box<dyn Error>> {
        let config = &*GLOBAL_CONFIG;
        let pool = MySqlPoolOptions::new()
            .max_connections(config.connection_num)
            .connect(&config.database)
            .await?;
        self.pool.replace(pool); // 存储连接池
        Ok(())
    }

    async fn connect(&self) -> Result<Self::Connection, Box<dyn Error>> {
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

    async fn connection_pool(&mut self) -> Result<(), Box<dyn Error>> {
        let config = &*GLOBAL_CONFIG;
        let pool = PgPoolOptions::new()
            .max_connections(config.connection_num)
            .connect(&config.database)
            .await?;
        self.pool.replace(pool);
        Ok(())
    }

    async fn connect(&self) -> Result<Self::Connection, Box<dyn Error>> {
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

pub async fn setup_database() -> Result<(), Box<dyn Error>> {
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

pub async fn get_db_connection() -> Result<DbConnection, Box<dyn Error>> {
    let config = &*GLOBAL_CONFIG;
    let db_manager = DB_MANAGER.get().ok_or("DB_MANAGER is not initialized")?;
    let conn = db_manager.read().await.connect().await?;
    match config.database_type.as_str() {
        "mysql" => {
            let mysql_conn = conn
                .downcast::<PoolConnection<MySql>>()
                .map_err(|_| "Failed to downcast to PoolConnection<MySql>")?;
            Ok(DbConnection::MySql(*mysql_conn))
        }
        "pgsql" => {
            let pg_conn = conn
                .downcast::<PoolConnection<Postgres>>()
                .map_err(|_| "Failed to downcast to PoolConnection<Postgres>")?;
            Ok(DbConnection::Postgres(*pg_conn))
        }
        _ => Err("Unsupported database type".into()),
    }
}

pub struct DBCrud;

impl DBCrud {
    /*
    example:

    #[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
    pub struct Model {
        pub id: String,
        pub object: String,
        pub created: i64,
        pub owned_by: String,
    }

    let test = Model {
        id: "test01".to_owned(),
        object: "test01".to_owned(),
        created: 1111,
        owned_by: "test01".to_owned(),
    };

    let _ = DBCrud::create("Models", &test).await;
     */
    pub async fn create<T: Serialize>(
        table_name: &str,
        record: &T,
    ) -> Result<(), Box<dyn Error>> {
        let conn = get_db_connection().await?;
        let dbtype = &*GLOBAL_CONFIG.database_type;
        let json_value = serde_json::to_value(record)?; // 序列化记录
        if let JsonValue::Object(map) = json_value {
            let columns: Vec<String> = map.keys().cloned().collect();
            let values: Vec<String> = if dbtype == "pgsql" {
                (1..=map.len()).map(|i| format!("${}", i)).collect() // PostgreSQL 使用 $1, $2, ...
            } else {
                map.values().map(|_| "?".to_string()).collect() // MySQL 使用 ?
            };

            let query_str = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table_name,
                columns.join(", "),
                values.join(", ")
            );
            // println!("Generated query: {}", query_str);
            match conn {
                DbConnection::MySql(mut mysql_conn) => {
                    let mut sql_query = query::<MySql>(&query_str);
                    for value in map.values() {
                        sql_query = Self::bind_value_query(sql_query, value);
                    }
                    sql_query.execute(&mut *mysql_conn).await?;
                }
                DbConnection::Postgres(mut pg_conn) => {
                    let mut sql_query = query::<Postgres>(&query_str);
                    for value in map.values() {
                        sql_query = Self::bind_value_query(sql_query, value);
                    }
                    sql_query.execute(&mut *pg_conn).await?;
                }
            }
        }

        Ok(())
    }

    /*
    example:

    use serde_json::json;

    #[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
    pub struct Model {
        pub id: String,
        pub object: String,
        pub created: i64,
        pub owned_by: String,
    }

    let id_value = json!("test01");
    let model = DBCrud::get::<Model>("models", "id", &id_value).await
     */
    pub async fn get<T: DeserializeOwned>(
        table_name: &str,
        id_column: &str, // 用于过滤行的列名，可以不是主键，如果存在列名不存在的情况，需要自行处理异常
        id_value: &JsonValue, // 用于过滤行的列值，使用 JsonValue 作为输入类型
    ) -> Result<Option<T>, Box<dyn Error>> 
    where
        T: for<'q> sqlx::FromRow<'q, sqlx::postgres::PgRow>
            + for<'q> sqlx::FromRow<'q, sqlx::mysql::MySqlRow>
            + DeserializeOwned
            + Send
            + Unpin,
    {
        let conn = get_db_connection().await?;
        let dbtype = &*GLOBAL_CONFIG.database_type;
    
        let query_str = format!(
            "SELECT * FROM {} WHERE {} = {}",
            table_name,
            id_column,
            if dbtype == "pgsql" { "$1" } else { "?" } // PostgreSQL 使用 $1，占位符
        );
    
        let result = match conn {
            DbConnection::MySql(mut mysql_conn) => {
                let mut sql_query = query_as::<_, T>(&query_str);
                sql_query = Self::bind_value_query_as(sql_query, id_value);
                sql_query.fetch_optional(&mut *mysql_conn).await?
            }
            DbConnection::Postgres(mut pg_conn) => {
                let mut sql_query = query_as::<_, T>(&query_str);
                sql_query = Self::bind_value_query_as(sql_query, id_value);
                sql_query.fetch_optional(&mut *pg_conn).await?
            }
        };
    
        Ok(result)
    }
    
    /*
    example:

    #[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
    pub struct Model {
        pub id: String,
        pub object: String,
        pub created: i64,
        pub owned_by: String,
    }

    let models = DBCrud::get_all::<Model>("Models").await;
     */
    pub async fn get_all<T: DeserializeOwned>(
        table_name: &str,
    ) -> Result<Vec<T>, Box<dyn Error>> 
    where
        T: for<'q> sqlx::FromRow<'q, sqlx::postgres::PgRow>
            + for<'q> sqlx::FromRow<'q, sqlx::mysql::MySqlRow>
            + DeserializeOwned
            + Send
            + Unpin,
    {
        let conn = get_db_connection().await?;
        let query_str = format!("SELECT * FROM {}", table_name);

        let result = match conn {
            DbConnection::MySql(mut mysql_conn) => {
                query_as::<_, T>(&query_str)
                    .fetch_all(&mut *mysql_conn)
                    .await?
            }
            DbConnection::Postgres(mut pg_conn) => {
                query_as::<_, T>(&query_str)
                    .fetch_all(&mut *pg_conn)
                    .await?
            }
        };

        Ok(result)
    }

    /*
    example:

    use serde_json::Value as JsonValue;

    #[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
    pub struct Model {
        pub id: String,
        pub object: String,
        pub created: i64,
        pub owned_by: String,
    }

    let updates = &[("owned_by", JsonValue::String("test02".to_owned()))];
    let conditions = &[("id", JsonValue::String("test01".to_owned()))];
    let rows_updated = DBCrud::update("models", updates, Some(conditions)).await;
     */
    pub async fn update<'q>(
        table_name: &str,
        updates: &[(&str, JsonValue)], // 更新字段和值
        conditions: Option<&[(&str, JsonValue)]>, // 更新条件，列名和对应的值，自行判断row不存在的情况
    ) -> Result<u64, Box<dyn Error>> {
        let conn = get_db_connection().await?;
        let dbtype = &*GLOBAL_CONFIG.database_type;
    
        let set_str: Vec<String> = updates
            .iter()
            .enumerate()
            .map(|(i, (col, _))| {
                if dbtype == "pgsql" {
                    format!("{} = ${}", col, i + 1)
                } else {
                    format!("{} = ?", col)
                }
            })
            .collect();
    
        let mut query_str = format!("UPDATE {} SET {}", table_name, set_str.join(", "));
    
        if let Some(conds) = conditions {
            let conds_str: Vec<String> = conds
                .iter()
                .enumerate()
                .map(|(i, (col, _))| {
                    if dbtype == "pgsql" {
                        format!("{} = ${}", col, updates.len() + i + 1)
                    } else {
                        format!("{} = ?", col)
                    }
                })
                .collect();
    
            if !conds_str.is_empty() {
                query_str.push_str(&format!(" WHERE {}", conds_str.join(" AND ")));
            }
        }
    
        let rows_affected = match conn {
            DbConnection::MySql(mut mysql_conn) => {
                let mut sql_query = query::<MySql>(&query_str);
    
                for (_, value) in updates {
                    sql_query = Self::bind_value_query(sql_query, value);
                }
                if let Some(conds) = conditions {
                    for (_, value) in conds {
                        sql_query = Self::bind_value_query(sql_query, value);
                    }
                }
    
                sql_query.execute(&mut *mysql_conn).await?.rows_affected()
            }
            DbConnection::Postgres(mut pg_conn) => {
                let mut sql_query = query::<Postgres>(&query_str);
    
                for (_, value) in updates {
                    sql_query = Self::bind_value_query(sql_query, value);
                }
                if let Some(conds) = conditions {
                    for (_, value) in conds {
                        sql_query = Self::bind_value_query(sql_query, value);
                    }
                }
    
                sql_query.execute(&mut *pg_conn).await?.rows_affected()
            }
        };
    
        Ok(rows_affected)
    }

    /*
    example:

    use serde_json::Value as JsonValue;

    #[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
    pub struct Model {
        pub id: String,
        pub object: String,
        pub created: i64,
        pub owned_by: String,
    }

    let conditions = &[("id", JsonValue::String("test01".to_owned()))];
    let rows_deleted = DBCrud::delete("models", Some(conditions)).await;
     */
    pub async fn delete<'q>(
        table_name: &str,
        conditions: Option<&[(&str, JsonValue)]>, // 删除条件
    ) -> Result<u64, Box<dyn Error>> {
        let conn = get_db_connection().await?;
        let dbtype = &*GLOBAL_CONFIG.database_type;
    
        let mut query_str = format!("DELETE FROM {}", table_name);
    
        if let Some(conds) = conditions {
            let conds_str: Vec<String> = conds
                .iter()
                .enumerate()
                .map(|(i, (col, _))| {
                    if dbtype == "pgsql" {
                        format!("{} = ${}", col, i + 1)
                    } else {
                        format!("{} = ?", col)
                    }
                })
                .collect();
    
            if !conds_str.is_empty() {
                query_str.push_str(&format!(" WHERE {}", conds_str.join(" AND ")));
            }
        }
    
        let rows_affected = match conn {
            DbConnection::MySql(mut mysql_conn) => {
                let mut sql_query = query::<MySql>(&query_str);
    
                if let Some(conds) = conditions {
                    for (_, value) in conds {
                        sql_query = Self::bind_value_query(sql_query, value);
                    }
                }
    
                sql_query.execute(&mut *mysql_conn).await?.rows_affected()
            }
            DbConnection::Postgres(mut pg_conn) => {
                let mut sql_query = query::<Postgres>(&query_str);
    
                if let Some(conds) = conditions {
                    for (_, value) in conds {
                        sql_query = Self::bind_value_query(sql_query, value);
                    }
                }
    
                sql_query.execute(&mut *pg_conn).await?.rows_affected()
            }
        };
    
        Ok(rows_affected)
    }

    fn bind_value_query<'q, DB>(
        sql_query: sqlx::query::Query<'q, DB, <DB as sqlx::Database>::Arguments<'q>>,
        value: &'q JsonValue,
    ) -> sqlx::query::Query<'q, DB, <DB as sqlx::Database>::Arguments<'q>>
    where
        DB: sqlx::Database,
        i8: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
        i32: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
        i64: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
        f64: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
        String: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
        bool: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
        Option<String>: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
    {
        match value {
            JsonValue::String(s) => sql_query.bind(s),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if DB::NAME == "mysql" {
                        sql_query.bind(i as i32) // MySQL 需要显式转换为 i32
                    } else {
                        sql_query.bind(i)
                    }
                } else if let Some(f) = n.as_f64() {
                    sql_query.bind(f)
                } else {
                    sql_query
                }
            }
            JsonValue::Bool(b) => {
                if DB::NAME == "mysql" {
                    sql_query.bind(*b as i8) // MySQL 布尔值用 TINYINT
                } else {
                    sql_query.bind(*b)
                }
            }
            _ => sql_query.bind(None::<String>),
        }
    }
    
    fn bind_value_query_as<'q, DB, T>(
        sql_query: sqlx::query::QueryAs<'q, DB, T, <DB as sqlx::Database>::Arguments<'q>>,
        value: &'q JsonValue,
    ) -> sqlx::query::QueryAs<'q, DB, T, <DB as sqlx::Database>::Arguments<'q>>
    where
        DB: sqlx::Database,
        T: Send + Unpin,
        i8: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
        i32: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
        i64: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
        f64: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
        String: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
        bool: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
        Option<String>: sqlx::Type<DB> + sqlx::Encode<'q, DB>,
    {
        match value {
            JsonValue::String(s) => sql_query.bind(s),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if DB::NAME == "mysql" {
                        sql_query.bind(i as i32)
                    } else {
                        sql_query.bind(i)
                    }
                } else if let Some(f) = n.as_f64() {
                    sql_query.bind(f)
                } else {
                    sql_query
                }
            }
            JsonValue::Bool(b) => {
                if DB::NAME == "mysql" {
                    sql_query.bind(*b as i8)
                } else {
                    sql_query.bind(*b)
                }
            }
            _ => sql_query.bind(None::<String>),
        }
    }
}
