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

use unicode_segmentation::UnicodeSegmentation;

pub fn is_valid_name(name: &str) -> bool {
    let is_empty = name.trim().is_empty();

    let too_long = name.graphemes(true).count() > 256;

    let forbidden_characters = ['/', ')', '(', ';', '"'];

    let contains_forbidden_characters = name
        .chars()
        .any(|c| { forbidden_characters.contains(&c) });
    println!("name {} is_empty = {}, too_long = {}, contains_forbidden = {}",
                     name,
                     is_empty,
                     too_long,
                     contains_forbidden_characters);
    is_empty || too_long || contains_forbidden_characters
}

#[tracing::instrument(
name = "Adding new subscriber",
skip(form, pool),
fields(
subscriber_id = % form.email,
subscriber_name = % form.name
)
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    Uuid::new_v4().to_string();

    if !is_valid_name(&form.name) {
        return HttpResponse::BadRequest().finish();
    }
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
