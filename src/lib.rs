use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer,HttpRequest};
use actix_web::http::{StatusCode};
use actix_web::body::Body;
use std::net::TcpListener;

pub mod configuration;
pub mod routes;
pub mod startup;

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
