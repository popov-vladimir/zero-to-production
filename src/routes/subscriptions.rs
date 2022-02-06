use actix_web::{web, HttpResponse};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::PgPool;
use uuid::Uuid;
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
name = "Saving new subscriber details in the database",
skip(new_subscriber, pool),
fields(
subscriber_id = % new_subscriber.email.as_ref(),
subscriber_name = % new_subscriber.name.as_ref()
)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    pool: &PgPool,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"insert into subscriptions(id, email, name, subscribed_at,status) 
        values($1,$2,$3,$4,'pending')"#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;
    Ok(subscriber_id)
}

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::ApplicationBaseUrl,
};

use std::convert::{TryFrom, TryInto};

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;

        let email = SubscriberEmail::parse(form.email)?;

        Ok(NewSubscriber { name, email })
    }
}

pub fn parse_subscriber(form: web::Form<FormData>) -> Result<NewSubscriber, String> {
    let name = SubscriberName::parse(form.0.name)?;

    let email = SubscriberEmail::parse(form.0.email)?;

    Ok(NewSubscriber { name, email })
}
#[tracing::instrument(
name = "Adding new subscriber",
skip(form, pool, email_client,base_url),
fields(
subscriber_id = % form.email,
subscriber_name = % form.name
)
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> HttpResponse {
    let new_subscriber: NewSubscriber = match form.0.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let subscriber_id = match insert_subscriber(&new_subscriber, &pool).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let subscription_token = generate_subscription_token();

    if store_token(subscriber_id, &subscription_token, &pool)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    };

    HttpResponse::Ok().finish()
}

pub async fn store_token(
    subscriber_id: Uuid,
    subscription_token: &str,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscriber_id, subscription_token) VALUES ($1,$2)"#,
        subscriber_id,
        subscription_token
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "sending confirmation email",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );

    let html_body = &format!(
        "Please, confirm your email <a href=\"{}\">here</a>",
        confirmation_link
    );
    let plain_body = &format!("Please, confirm your email \n {}", confirmation_link);

    email_client
        .send_email(
            new_subscriber.email,
            "welcome to our newsletter",
            html_body,
            plain_body,
        )
        .await
    // Ok(())
}
