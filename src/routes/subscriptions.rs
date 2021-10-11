use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use tracing_futures::Instrument;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
name = "Adding new subscriber",
skip(_form, _pool),
fields(
request_id = % Uuid::new_v4(),
subscriber_id = % _form.email,
subscriber_name = % _form.name
)
)]
pub async fn subscribe(_form: web::Form<FormData>, _pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4().to_string();

    let query_span = tracing::info_span!("Saving new subscriber details in the database");
    match sqlx::query!(
        r#"insert into subscriptions(id, email, name, subscribed_at) values($1,$2,$3,$4)"#,
        Uuid::new_v4(),
        _form.email,
        _form.name,
        Utc::now()
    )
        .execute(_pool.get_ref())
        .instrument(query_span)
        .await
    {
        Ok(_) => {
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("request_id {}: failed to execute query! {:?}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
