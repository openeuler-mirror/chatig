use actix_web::{web, HttpResponse, Error};
use actix_web::error::{ErrorInternalServerError, ErrorBadRequest};
use async_trait::async_trait;
use crate::cores::control::services::ServiceManager;
use crate::cores::models::generate::generate_controller::{Generate, GenerateRequest};
use crate::cores::models::generate::generate_utils::{
    generate_response_stream,
    generate_response_non_stream,
    get_generate_request_body,
    get_generate_response,
};
use crate::cores::models::chat::chat_utils::RequestInfo;

pub struct StdGenerateModel {
    pub active_model: String,
}

#[async_trait]
impl Generate for StdGenerateModel {
    async fn generate(
        &self,
        req_body: web::Json<GenerateRequest>,
        user_name: String,
        user_id: String,
        service_name: String,
        service_id: String,
        user_type: String,
        user_level: String,
        pay_status: String,
        aicpid: String,
        is_stream: bool
    ) -> Result<HttpResponse, Error> {
        // 1. Read the model's parameter configuration
        let req_model_name = req_body.model.clone();

        let service_manager = ServiceManager::default();
        let service_option = service_manager.get_service_by_model(&self.active_model, None, aicpid).await?;
        let mut service = match service_option {
            Some(svc) => svc,
            None => return Err(ErrorBadRequest(format!("{} model is not supported", req_model_name))),
        };

        // 2. 构建模型请求
        let request_body = get_generate_request_body(req_body)
            .await
            .map_err(|err| ErrorInternalServerError(err.to_string()))?;

        // 3. 获取模型响应
        let (response, start_time) = match get_generate_response(request_body, &mut service,is_stream).await {
            Ok((resp, start_time)) => (resp, start_time),
            Err(err) => return Err(ErrorInternalServerError(err.to_string())),
        };

        // 4. Return the response based on the request's streaming status
        let req_info = RequestInfo {
            req_model_name,
            user_name,
            user_id,
            service_name: service_name.clone(),
            service_id: service_id.clone(),
            start_time,
            user_type: user_type.clone(),
            user_level: user_level.clone(),
            pay_status: pay_status.clone(),
            return_usage: false,
            instance_id: "generate_test".to_string(),
        };

        //将模型返回的数据转化为适合客户端的格式
        if is_stream {
            generate_response_stream(response, req_info).await
        } else {
            generate_response_non_stream(response, req_info).await
        }
    }
}