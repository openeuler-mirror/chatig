#[cfg(test)]
pub mod tests {
    use actix_web::{test, App};
    use crate::apis::models_api::chat::health;
    use crate::apis::models_api::schemas::{EmbeddingRequest, EmbeddingResponse};


    #[actix_rt::test]
    async fn test_health() {
        let mut app = test::init_service(App::new().service(health)).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        assert_eq!(body, "OK");
    }

    // test /v1/embeddings api
    #[actix_rt::test]
    async fn test_v1_embeddings() {
        let mut app = test::init_service(
            App::new().service(
                crate::apis::models_api::embeddings::v1_embeddings
            )
        ).await;

        let request_body = EmbeddingRequest {
            input: vec!["This is a test input".to_string()],
            model: "text-embedding-ada-002".to_string(),
            encoding_format: None,
            dimensions: None,
            user: None
        };
        let req = test::TestRequest::post()
          .uri("/v1/embeddings")
          .set_json(&request_body)
          .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let result: EmbeddingResponse = serde_json::from_slice(&body).expect("Failed to deserialize response");
        assert_eq!(result.model, "text-embedding-ada-002");
    }
}