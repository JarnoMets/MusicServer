use actix_web::{test, App, web};
use music_server::routes;

#[actix_web::test]
async fn test_health_check() {
    let app = test::init_service(
        App::new().route("/health", web::get().to(routes::health_check))
    ).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "healthy");
}
