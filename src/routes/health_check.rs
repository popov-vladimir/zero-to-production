use actix_web::HttpResponse;

pub async fn health_check() -> HttpResponse {
    tracing::debug!("health_check received request");
    HttpResponse::Ok().finish()
}
