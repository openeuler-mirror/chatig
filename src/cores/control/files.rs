use std::error::Error;

use crate::meta::files::traits::{File, FilesTrait};
use crate::meta::files::impls::FilesImpl;


pub struct FileManager {
    files: Box<dyn FilesTrait>,
}

// Default implementation for FileManager
impl Default for FileManager {
    fn default() -> Self {
        FileManager {
            files: Box::new(FilesImpl),
        }
    }
}

impl FileManager{
    pub fn _new(files: Box<dyn FilesTrait>) -> Self {
        FileManager { files }
    }

    pub async fn add_file_object(&self, file: File) -> Result<(), Box<dyn Error>> {
        self.files.add_file_object(file).await
    }

    pub async fn delete_file_object(&self, file_id: &str) -> Result<(), Box<dyn Error>> {
        self.files.delete_file_object(file_id).await
    }

    pub async fn update_file_object(&self, file: File) -> Result<u64, Box<dyn Error>> {
        self.files.update_file_object(file).await
    }

    pub async fn get_file_object(&self, file_id: &str) -> Result<Option<File>, Box<dyn Error>> {
        self.files.get_file_object(file_id).await
    }

    pub async fn get_all_file_objects(&self) -> Result<Vec<File>, Box<dyn Error>> {
        self.files.get_all_file_objects().await
    }
}