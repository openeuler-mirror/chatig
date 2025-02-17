use serde_yaml;
use serde_json::json;
use std::fs;
use std::error::Error;
use async_trait::async_trait;
use rand::Rng;

use crate::meta::services::traits::{Services, ModelsService, ServiceConfig, ServicesTrait};
use crate::meta::connection::DBCrud;

pub struct ServicesImpl;

#[async_trait]
impl ServicesTrait for ServicesImpl {
    /// 加载/etc/chatig/services.yaml文件到 `services` 和 `models_service` 表中
    async fn load_services_table(&self) -> Result<(), Box<dyn Error>> {
        let yaml_content = fs::read_to_string("/etc/chatig/services.yaml").map_err(|err| {
            eprintln!("Failed to read services YAML file: {}", err);
            err
        })?;
        
        let services: Vec<ServiceConfig> = serde_yaml::from_str(&yaml_content).map_err(|err| {
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
            });
    
            if let Err(err) = DBCrud::create("services", &service_data).await {
                eprintln!("Failed to insert service: {}", err);
                continue;
            }
    
            // 将模型数据插入到 models_service 表
            for model in service.models {
                let model_data = json!({
                    "serviceid": service.id,
                    "modelid": model
                });
    
                if let Err(err) = DBCrud::create("models_service", &model_data).await {
                    eprintln!("Failed to insert model (serviceid: {}, modelid: {}): {}", service.id, model, err);
                    continue;
                }
            }
        }
    
        Ok(())
    }

    /// 将 `ServiceConfig` 插入到 `services` 和 `models_service` 表中
    async fn create_service(&self, service: &ServiceConfig) -> Result<(), Box<dyn Error>> {
        // 插入到 `services` 表
        let service_data = json!({
            "id": service.id,
            "servicetype": service.servicetype,
            "status": service.status,
            "url": service.url,
            "model_name": service.model_name,
            "active_model": service.active_model,
        });
        DBCrud::create("services", &service_data).await?;

        // 插入到 `models_service` 表
        for model in &service.models {
            let model_data = json!({
                "serviceid": service.id,
                "modelid": model
            });
            DBCrud::create("models_service", &model_data).await?;
        }

        Ok(())
    }

    /// 删除 `services` 表中的记录，同时级联删除 `models_service` 表中的相关记录
    async fn delete_service(&self, service_id: &str) -> Result<u64, Box<dyn Error>> {
        // 删除 `models_service` 中相关记录
        let model_conditions = &[("serviceid", json!(service_id))];
        let delete_num = DBCrud::delete("models_service", Some(model_conditions)).await?;

        // 删除 `services` 中的记录
        let service_conditions = &[("id", json!(service_id))];
        DBCrud::delete("services", Some(service_conditions)).await?;

        Ok(delete_num)
    }

    /// 更新 `services` 表中的记录，但不会修改 `models_service` 中的模型信息
    async fn update_service(&self, service: &ServiceConfig) -> Result<u64, Box<dyn Error>> {
        let updates = &[
            ("servicetype", json!(service.servicetype)),
            ("status", json!(service.status)),
            ("url", json!(service.url)),
            ("model_name", json!(service.model_name)),
            ("active_model", json!(service.active_model)),
        ];
        let conditions = &[("id", json!(service.id))];
        let rows_updated = DBCrud::update("services", updates, Some(conditions)).await?;

        Ok(rows_updated)
    }

    /// 根据服务 ID 查询 `ServiceConfig`
    async fn get_service(&self, service_id: &str) -> Result<Option<ServiceConfig>, Box<dyn Error>> {
        // 查询 `services` 表中的记录
        let service: Option<Services> = DBCrud::get("services", "id", &json!(service_id)).await?;
    
        if let Some(service) = service {
            // 查询 `models_service` 表中的模型列表
            let models: Vec<ModelsService> = DBCrud::get_all("models_service").await?;
    
            // 提取模型 ID 列表
            let model_ids = models
                .iter()
                .filter_map(|record| Some(record.modelid.clone()))
                .collect::<Vec<String>>();
    
            // 组装成完整的 `ServiceConfig`
            let service_config = ServiceConfig {
                id: service.id,
                servicetype: service.servicetype,
                status: service.status,
                url: service.url,
                model_name: service.model_name,
                active_model: service.active_model,
                models: model_ids,
            };
    
            Ok(Some(service_config))
        } else {
            Ok(None)
        }
    }

    /// 根据模型名称查询 `ServiceConfig`
    async fn get_service_by_model(&self, active_model: &str) -> Result<Option<ServiceConfig>, Box<dyn Error>>{
        // 查询 `services` 表中的记录
        let services: Vec<Services> = DBCrud::get_multis("services", "active_model", &json!(active_model)).await?;
        let service_num = services.len();

        if service_num > 0 {
            let service = if services.len() > 1 {
                get_random_service(&services).unwrap_or_else(|| services[0].clone())
            } else {
                services[0].clone()
            };
            
            // 组装成完整的 `ServiceConfig`
            let service_config = ServiceConfig {
                id: service.id,
                servicetype: service.servicetype,
                status: service.status,
                url: service.url,
                model_name: service.model_name,
                active_model: service.active_model,
                models: vec![String::from("")],
            };
    
            Ok(Some(service_config))
        } else {
            Ok(None)
        }
    }

    /// 查询所有 `ServiceConfig`
    async fn get_all_services(&self) -> Result<Vec<ServiceConfig>, Box<dyn Error>> {
        // 查询所有 `services` 表中的记录
        let services: Vec<Services> = DBCrud::get_all("services").await?;
    
        // 查询所有 `models_service` 表中的记录
        let models: Vec<ModelsService> = DBCrud::get_all("models_service").await?;
    
        // 为每个服务填充模型信息
        let mut service_configs = Vec::new();
        for service in services {
            // 找到与当前服务关联的模型
            let model_ids = models
                .iter()
                .filter(|record| record.serviceid == service.id)
                .map(|record| record.modelid.clone())
                .collect::<Vec<String>>();
    
            // 组装成 `ServiceConfig`
            let service_config = ServiceConfig {
                id: service.id,
                servicetype: service.servicetype,
                status: service.status,
                url: service.url,
                model_name: service.model_name,
                active_model: service.active_model,
                models: model_ids,
            };
    
            service_configs.push(service_config);
        }
    
        Ok(service_configs)
    }
    
}

fn get_random_service(services: &Vec<Services>) -> Option<Services> {
    let n = services.len();

    // 随机生成一个索引
    let index = rand::thread_rng().gen_range(0..n);

    // 返回随机服务
    services.get(index).cloned()
} 