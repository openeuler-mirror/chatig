use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;
use serde::{Serialize, Deserialize};

// file_object table structure
#[derive(Serialize, Deserialize, Debug)]
pub struct FileObject {
    pub id: i32,           // 使用 i32 类型以匹配数据库表的 ID 类型
    pub object: String,
    pub bytes: i32,
    pub created_at: i64,
    pub filename: String,
    pub purpose: String,
}

// add file object
pub async fn add_file_object(
    pool: &Pool<PostgresConnectionManager<NoTls>>,
    file_object: FileObject,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let query = "
        INSERT INTO file_object (object, bytes, created_at, filename, purpose)
        VALUES ($1, $2, $3, $4, $5)";
    
    client.execute(query, &[&file_object.object, &file_object.bytes, &file_object.created_at, &file_object.filename, &file_object.purpose]).await?;

    Ok(())
}

// delete file object
pub async fn delete_file_object(
    pool: &Pool<PostgresConnectionManager<NoTls>>,
    file_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let query = "DELETE FROM file_object WHERE id = $1";
    
    client.execute(query, &[&file_id]).await?;

    Ok(())
}

// update file object
pub async fn _update_file_object(
    pool: &Pool<PostgresConnectionManager<NoTls>>,
    file_object: FileObject,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let query = "
        UPDATE file_object
        SET object = $1, bytes = $2, created_at = $3, filename = $4, purpose = $5
        WHERE id = $6";
    
    client.execute(query, &[
        &file_object.object,
        &file_object.bytes,
        &file_object.created_at,
        &file_object.filename,
        &file_object.purpose,
        &file_object.id,
    ]).await?;

    Ok(())
}

// get file object
pub async fn get_file_object_by_id(
    pool: &Pool<PostgresConnectionManager<NoTls>>,
    file_id: i32,
) -> Result<Option<FileObject>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let query = "SELECT id, object, bytes, created_at, filename, purpose FROM file_object WHERE id = $1";

    if let Some(row) = client.query_opt(query, &[&file_id]).await? {
        let file_object = FileObject {
            id: row.get(0),
            object: row.get(1),
            bytes: row.get(2),
            created_at: row.get(3),
            filename: row.get(4),
            purpose: row.get(5),
        };

        Ok(Some(file_object))
    } else {
        Ok(None)
    }
}

// list all file objects
pub async fn list_file_objects(
    pool: &Pool<PostgresConnectionManager<NoTls>>,
) -> Result<Vec<FileObject>, Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let query = "SELECT id, object, bytes, created_at, filename, purpose FROM file_object";

    let mut file_objects = Vec::new();
    for row in client.query(query, &[]).await? {
        let file_object = FileObject {
            id: row.get(0),
            object: row.get(1),
            bytes: row.get(2),
            created_at: row.get(3),
            filename: row.get(4),
            purpose: row.get(5),
        };

        file_objects.push(file_object);
    }

    Ok(file_objects)
}


