use actix_web::{post, web, Error, HttpResponse, Responder};
use actix_web::error::ErrorBadRequest;

use crate::apis::models_api::schemas::{ImageGenerationRequest, ImageGenerationResponse};
use crate::apis::schemas::ErrorResponse;
use crate::cores::image_models::image_controller::ImageProvider;
use crate::cores::image_models::sdxl::SdxlTurbo;
// use crate::cores::image_models::stable_diffusion::StableDiffusion;

// Configure the actix_web service routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(v1_images_generations);
}

// define an interface layer that calls the image generation method of the large model
struct IMG {
    model: Box<dyn ImageProvider>,
}

impl IMG {
    fn new(model: Box<dyn ImageProvider>) -> Self {
        IMG { model }
    }

    async fn image_provider(&self, req_body: web::Json<ImageGenerationRequest>) -> Result<ImageGenerationResponse, String> {
        self.model.image_provider(req_body).await
    }
}

#[utoipa::path(
    post,  // 请求方法
    path = "/v1/images/generations",  // 路径
    request_body = ImageGenerationRequest,
    responses(
        (status = 200, body = ImageGenerationResponse),
        (status = 400, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    )  // 响应内容
)]

// Handle the POST request for /v1/images/generations.
#[post("/v1/images/generations")]
async fn v1_images_generations(req_body: web::Json<ImageGenerationRequest>) -> Result<impl Responder, Error> {
    // 1. Validate the required fields.
    if req_body.prompt.is_empty() || req_body.model.is_empty() {
        let error_response = ErrorResponse {
            error: "Invalid request: prompt and model are required fields".into(),
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // 2. Call the underlying API and return a unified data format
    let model_name = req_body.model.clone();
    let model_series = model_name.split("/").next().unwrap_or("");
    let model: IMG = match model_series {
        "sdxl-turbo" => IMG::new(Box::new(SdxlTurbo {})),
        // "stable-diffusion-v1.5" => IMG::new(Box::new(StableDiffusion {})),
        _ => return Err(ErrorBadRequest(format!("Unsupported {} model series!", model_series))),
    };

    // 3. Send the request to the model service
    let response = model.image_provider(req_body).await;
    match response {
        Ok(resp) => Ok(HttpResponse::Ok().json(resp)),
        Err(err) => {
            let error_response = ErrorResponse {
                error: format!("Failed to get response from {} image generation: {}", model_name, err),
            };
            Ok(HttpResponse::InternalServerError().json(error_response))
        }
    }
}