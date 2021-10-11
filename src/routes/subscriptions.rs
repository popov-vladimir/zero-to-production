use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
name = "Saving new subscriber details in the database",
skip(form, pool),
fields(
request_id = % Uuid::new_v4(),
subscriber_id = % form.email,
subscriber_name = % form.name
)
)]
pub async fn insert_subscriber(form: &FormData, pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into subscriptions(id, email, name, subscribed_at) values($1,$2,$3,$4)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query {:?}", e);
            e
        })?;
    Ok(())
}

#[tracing::instrument(
name = "Adding new subscriber",
skip(form, pool),
fields(
request_id = % Uuid::new_v4(),
subscriber_id = % form.email,
subscriber_name = % form.name
)
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    Uuid::new_v4().to_string();

    match insert_subscriber(&form, &pool).await
    {
        Ok(_) => {
            HttpResponse::Ok().finish()
        }
        Err(_) => {
            HttpResponse::InternalServerError().finish()
        }
    }
}
