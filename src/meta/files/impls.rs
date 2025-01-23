use serde_json::json;
use std::error::Error;
use async_trait::async_trait;

use crate::meta::files::traits::{File, FilesTrait};
use crate::meta::connection::DBCrud;

pub struct FilesImpl;

#[async_trait]
impl FilesTrait for FilesImpl {
    // add file object
    async fn add_file_object(&self, file: File) -> Result<(), Box<dyn Error>>{
        // 插入到 `files` 表中
        let file_object = json!({
            "id": file.id,
            "object": file.object,
            "bytes": file.bytes,
            "created_at": file.created_at,
            "filename": file.filename,
            "purpose": file.purpose,
        });
        DBCrud::create("files", &file_object).await?;

        Ok(())
    }

    // delete file object
    async fn delete_file_object(&self, file_id: &str) -> Result<(), Box<dyn Error>>{
        // 删除 `files` 表中的相关的记录数据
        let file_conditions = &[("id", json!(file_id))];
        DBCrud::delete("files", Some(file_conditions)).await?;

        Ok(())
    }

    // update file object
    async fn update_file_object(&self, file: File) -> Result<u64, Box<dyn Error>>{
        let updates = &[
            ("object", json!(file.object)),
            ("bytes", json!(file.bytes)),
            ("filename", json!(file.filename)),
            ("purpose", json!(file.purpose)),
        ];

        let conditions = &[("id", json!(file.id))];
        let rows_updated = DBCrud::update("files", updates, Some(conditions)).await?;

        Ok(rows_updated)
    }

    // get file object
    async fn get_file_object(&self, file_id: &str) -> Result<Option<File>, Box<dyn Error>>{
        let file: Option<File> = DBCrud::get("files", "id", &json!(file_id)).await?;

        Ok(file)
    }

    async fn get_all_file_objects(&self) -> Result<Vec<File>, Box<dyn Error>>{
        let files: Vec<File> = DBCrud::get_all("files").await?;

        Ok(files)
    }
}