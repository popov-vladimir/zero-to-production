use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(_form: web::Form<FormData>, _pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4().to_string();

    log::info!("request_id {}: new subscriber! {} {}",request_id, _form.name, _form.email);
    match sqlx::query!(
        r#"insert into subscriptions(id, email, name, subscribed_at) values($1,$2,$3,$4)"#,
        Uuid::new_v4(),
        _form.email,
        _form.name,
        Utc::now()
    )
    .execute(_pool.get_ref())
    .await
    {
        Ok(_) => {
            log::info!("request_id {}: new subscriber successfully saved", request_id);
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            log::error!("request_id {}: failed to execute query! {:?}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
