use crate::meta::models::traits::{ModelTrait, Model};
use crate::meta::models::impls::ModelImpl;

pub struct ModelManager {
    models: Box<dyn ModelTrait>,
}

impl Default for ModelManager {
    fn default() -> Self {
        ModelManager {
            models: Box::new(ModelImpl),
        }
    }
}

impl ModelManager {
    pub fn _new(models: Box<dyn ModelTrait>) -> Self {
        ModelManager { models }
    }

    pub async fn get_models(&self) -> Result<Vec<Model>, Box<dyn std::error::Error>> {
        self.models.get_models().await
    }

    pub async fn get_model(&self, model_name: &str) -> Result<Option<Model>, Box<dyn std::error::Error>> {
        self.models.get_model(model_name).await
    }
}