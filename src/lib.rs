
pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;
pub mod domain;
pub mod email_client;

// #[derive(serde::Deserialize)]
// struct FormData {
//     email: String,
//     name: String
// }

// async fn health_check() -> HttpResponse {
//     HttpResponse::Ok().finish()
// }

// async fn greet(req: HttpRequest) -> HttpResponse {
//     let name = req.match_info().get("name").unwrap_or("world");
//     HttpResponse::with_body(StatusCode::OK, Body::from(format!("hello,{}",name)))
// }

// async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
//     HttpResponse::Ok().finish()
// }
