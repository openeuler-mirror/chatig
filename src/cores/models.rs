use crate::meta::connection::{get_db_connection,DbConnection};
use crate::apis::control_api::schemas::Model;
use sqlx::Row;


pub async fn get_models() -> Result<Vec<Model>, Box<dyn std::error::Error>> {
    let db_conn = get_db_connection().await?;
    let mut modelsinfo = Vec::new();
    match db_conn {
        // 处理 MySQL 数据库
        DbConnection::MySql(mut conn) => {
            let rows = sqlx::query("SELECT id, object, created, owned_by FROM models")
                .fetch_all(&mut *conn)
                .await?;
            for row in rows {
                let model = Model {
                    id: row.get("id"),
                    object: row.get("object"),
                    created: row.get("created"),
                    owned_by: row.get("owned_by"),
                };
                modelsinfo.push(model);
            }
            Ok(modelsinfo)
        }
        // 处理 PostgreSQL 数据库
        DbConnection::Postgres(mut conn) => {
            let rows = sqlx::query("SELECT id, object, created, owned_by FROM models")
                .fetch_all(&mut *conn)
                .await?;
            for row in rows {
                let model = Model {
                    id: row.get("id"),
                    object: row.get("object"),
                    created: row.get("created"),
                    owned_by: row.get("owned_by"),
                };
                modelsinfo.push(model);
            }
            Ok(modelsinfo)
        }
    }
}

pub async fn get_model(model_name: &str) -> Result<Option<Model>, Box<dyn std::error::Error>> {
    // 获取数据库连接
    let db_conn = get_db_connection().await?;

    match db_conn {
        DbConnection::MySql(mut conn) => {
            let row = sqlx::query("SELECT id, object, created, owned_by FROM models WHERE id = ?")
                .bind(model_name) // 绑定模型名称
                .fetch_optional(&mut *conn)
                .await?;

            if let Some(row) = row {
                Ok(Some(Model {
                    id: row.get("id"),
                    object: row.get("object"),
                    created: row.get("created"),
                    owned_by: row.get("owned_by"),
                }))
            } else {
                Ok(None) // 如果未找到，返回 None
            }
        }
        DbConnection::Postgres(mut conn) => {
            let row = sqlx::query("SELECT id, object, created, owned_by FROM models WHERE id = $1")
                .bind(model_name)
                .fetch_optional(&mut *conn)
                .await?;

            if let Some(row) = row {
                Ok(Some(Model {
                    id: row.get("id"),
                    object: row.get("object"),
                    created: row.get("created"),
                    owned_by: row.get("owned_by"),
                }))
            } else {
                Ok(None)
            }
        }
    }
}