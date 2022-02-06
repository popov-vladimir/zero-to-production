use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

use crate::email_client::EmailClient;

#[derive(serde::Deserialize)]
pub struct Parameters {
    pub subscription_token: String,
}

#[tracing::instrument(
    name = "confirmation subscription",
    skip(pool, _email_client, parameters),
    fields()
)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
    _email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let token: &str = &parameters.subscription_token;

    let subscriber_id = match get_subscriber_by_token(&pool,token).await {
        Err(_) => return HttpResponse::InternalServerError().finish(),
        Ok(subscriber_id) => subscriber_id 
    };

    match subscriber_id {
        None => return HttpResponse::Unauthorized().finish(),
        Some(subscriber_id) => if confirm_subscriber(subscriber_id, &pool).await.is_err() {
            return HttpResponse::InternalServerError().finish();
        }
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Marking subscriber as confirmed",
    skip(subscriber_id, pool)
)]
pub async fn confirm_subscriber(subscriber_id: Uuid, pool: &PgPool) -> Result<(), sqlx::Error> {

    sqlx::query!(r#"UPDATE subscriptions SET status = 'confirmed' where id = $1"#, subscriber_id)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("failed to confirm subscriber {}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Getting subscriber by token",
    skip(token, pool)
)]
pub async fn get_subscriber_by_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let query_result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("failed to retrieve subscriber {}", e);
        e
    })?;

    Ok(query_result.map(|r| r.subscriber_id))
}
