use chrono::Utc;
use sqlx::PgConnection;
use actix_web::{web, HttpResponse};
use uuid::Uuid;
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(_form: web::Form<FormData>,
     _connection: web::Data<PgConnection>) -> HttpResponse {
    
    sqlx::query!(r#"insert into subscriptions(id, email, name, subscribed_at) values($1,$2,$3,$4)"#,
    Uuid::new_v4(), _form.email, _form.name, Utc::now()
)
    .execute(_connection.get_ref())
    .expect("failed to insert subscription");
        HttpResponse::Ok().finish()
}
