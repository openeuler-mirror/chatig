use serde_yaml;
use serde_json::json;
use std::fs;
use std::error::Error;
use async_trait::async_trait;
use rand::Rng;
use log;

use crate::meta::services::traits::{Services, ServicesTrait};
use crate::meta::connection::DBCrud;

pub struct ServicesImpl;

#[async_trait]
impl ServicesTrait for ServicesImpl {
    /// 加载/etc/chatig/services.yaml文件到 `services` 表中
    async fn load_services_table(&self) -> Result<(), Box<dyn Error>> {
        let yaml_content = fs::read_to_string("/etc/chatig/services.yaml").map_err(|err| {
            eprintln!("Failed to read services YAML file: {}", err);
            err
        })?;
        
        let services: Vec<Services> = serde_yaml::from_str(&yaml_content).map_err(|err| {
            eprintln!("Failed to parse services YAML file: {}", err);
            err
        })?;
        
        // 遍历服务配置并插入到数据库中
        for service in services {
            // 将服务数据插入到 services 表
            let service_data = json!({
                "id": service.id,
                "servicetype": service.servicetype,
                "status": service.status,
                "url": service.url,
                "model_name": service.model_name,
                "active_model": service.active_model,
                "api_key": service.api_key,
                "context_length": service.context_length,
            });
    
            if let Err(err) = DBCrud::create("services", &service_data).await {
                eprintln!("Failed to insert service: {}", err);
                continue;
            }
        }
    
        Ok(())
    }

    /// 创建新的服务配置
    async fn create_service(&self, service: &Services) -> Result<(), Box<dyn Error>> {
        // 插入到 `services` 表
        let service_data = json!({
            "id": service.id,
            "servicetype": service.servicetype,
            "status": service.status,
            "url": service.url,
            "model_name": service.model_name,
            "active_model": service.active_model,
            "api_key": service.api_key,
            "context_length": service.context_length,
            "tags": service.tags,
        });
        DBCrud::create("services", &service_data).await?;

        Ok(())
    }

    /// 删除 `services` 表中的记录
    async fn delete_service(&self, service_id: &str) -> Result<u64, Box<dyn Error>> {
        // 删除 `services` 中的记录
        let service_conditions = &[("id", json!(service_id))];
        let delete_num = DBCrud::delete("services", Some(service_conditions)).await?;

        Ok(delete_num)
    }

    /// 更新 `services` 表中的记录
    async fn update_service(&self, service: &Services) -> Result<u64, Box<dyn Error>> {
        let updates = &[
            ("servicetype", json!(service.servicetype)),
            ("status", json!(service.status)),
            ("url", json!(service.url)),
            ("model_name", json!(service.model_name)),
            ("active_model", json!(service.active_model)),
            ("api_key", json!(service.api_key)),
            ("context_length", json!(service.context_length)),
            ("tags", json!(service.tags))
            
        ];
        let conditions = &[("id", json!(service.id))];
        let rows_updated = DBCrud::update("services", updates, Some(conditions)).await?;

        Ok(rows_updated)
    }

    /// 根据服务 ID 查询 `Services`
    async fn get_service(&self, service_id: &str) -> Result<Option<Services>, Box<dyn Error>> {
        // 查询 `services` 表中的记录
        let service: Option<Services> = DBCrud::get("services", "id", &json!(service_id)).await?;

        Ok(service)
    }

    /// Query `Services` by model name
    async fn get_service_by_model(&self, active_model: &str, input_token: Option<f32>, aicpid_json: String) -> Result<Option<Services>, Box<dyn Error>> {
        // Set default value for input_tokens if not provided
        let input_token = input_token.unwrap_or(0.0);
        // Deserialization aicp_id
        let aicpid: Vec<String> = match serde_json::from_str(&aicpid_json) {
            Ok(v) => v,
            Err(e) => {
                log::warn!(target: "access_log", "aicpid deserialization failed: {} (raw data: {})", e, aicpid_json);
                Vec::new()
            }
        };

        // Query records from `services` table
        let services: Vec<Services> = DBCrud::get_multis("services", "active_model", &json!(active_model)).await?;
        
        // Filter services based on status
        let active_services: Vec<Services> = services.into_iter()
            .filter(|service| service.status == "active")
            .collect();
        if active_services.is_empty() {
            return Err(format!("No active services found for model '{}'", active_model).into());
        }

        // Further filter services based on input_token
        let active_services: Vec<Services> = active_services.into_iter()
            .filter(|service| service.context_length >= input_token)
            .collect();
        if active_services.is_empty() {
            return Err(format!("No services found for model '{}' that can handle {} input tokens", active_model, input_token).into());
        }

        // Further filter services based on aicp_id
        let aicpid_filter = |service: &Services| -> bool {
            // 空数组不过滤 | 非空数组时检查包含关系
            aicpid.is_empty() || aicpid.contains(&service.id)
        };
        let active_services: Vec<Services> = active_services.into_iter()
            .filter(aicpid_filter)
            .collect();
        if active_services.is_empty() {
            return Err(format!("No services found for model '{}' that can match aicp_id: {:?}", active_model, aicpid).into());
        }

        log::info!(target: "access_log", " services: {:?} ||| aicpid列表: {:?}", active_services, aicpid);//////////////////////

        // Select a random service if multiple are available
        let service = if active_services.len() > 1 {
            get_service_by_context_length(&active_services).unwrap_or_else(|| active_services[0].clone())
        } else {
            active_services[0].clone()
        };

        Ok(Some(service))
    }

    /// 查询所有 `Services`
    async fn get_all_services(&self) -> Result<Vec<Services>, Box<dyn Error>> {
        // 查询所有 `services` 表中的记录
        let services: Vec<Services> = DBCrud::get_all("services").await?;
    
        Ok(services)
    }

    // /// Query `Services` by model name to get the corresponding id
    // async fn get_id_by_model(&self, model_name: &str) -> Result<String, Box<dyn Error>> {
    //     // Query records from `services` table
    //     let services: Vec<Services> = DBCrud::get_multis("services", "model_name", &json!(model_name)).await?;
        
    //     // Filter services based on status
    //     let active_services: Vec<Services> = services.into_iter()
    //         .filter(|service| service.status == "active")
    //         .collect();
        
    //     if active_services.is_empty() {
    //         return Err(format!("No active services found for model name '{}'", model_name).into());
    //     }

    //     // Select the first active service (or you could modify this to select randomly like in the original)
    //     let service = active_services[0].clone();
        
    //     Ok(service.id)
    // }
    
}

fn get_random_service(services: &[Services]) -> Option<Services> {    
    // Generate a random index
    let index = rand::thread_rng().gen_range(0..services.len());
    
    // Return the randomly selected service
    services.get(index).cloned()
}

fn get_service_by_context_length(services: &[Services]) -> Option<Services> {
    // find the service with the minimum context_length
    let mut min_service = services[0].clone();
    let mut min_services = Vec::new();
    
    for service in services.iter() {
        if service.context_length < min_service.context_length {
            min_service = service.clone();
            min_services.clear();
            min_services.push(service.clone());
        } else if service.context_length == min_service.context_length {
            min_services.push(service.clone());
        }
    }
        
    if min_services.len() > 1 {
        get_random_service(&min_services)
    } else {
        Some(min_services[0].clone())
    }
}