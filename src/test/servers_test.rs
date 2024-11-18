#[cfg(test)]
pub mod tests {
    use actix_web::{test, App};
    use crate::servers::server::health;

    #[actix_rt::test]
    async fn test_health() {
        let mut app = test::init_service(App::new().service(health)).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        assert_eq!(body, "OK");
    }
}