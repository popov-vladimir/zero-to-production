use actix_http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[derive(Debug)]
pub enum SubscribeError {
    ValidationError(String),
    DatabaseError(sqlx::Error),
    StoreTokenError(StoreTokenError),
    SendEmailError(reqwest::Error),
}
impl From<reqwest::Error> for SubscribeError {
    fn from(e: reqwest::Error) -> Self {
        Self::SendEmailError(e)
    }
}
impl From<sqlx::Error> for SubscribeError {
    fn from(e: sqlx::Error) -> Self {
        Self::DatabaseError(e)
    }
}
impl From<StoreTokenError> for SubscribeError {
    fn from(e: StoreTokenError) -> Self {
        Self::StoreTokenError(e)
    }
}
impl From<String> for SubscribeError {
    fn from(e: String) -> Self {
        Self::ValidationError(e)
    }
}
impl std::fmt::Display for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Adding a subscriber has failed")
    }
}
pub struct StoreTokenError(sqlx::Error);

impl std::error::Error for SubscribeError {}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> actix_http::StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::DatabaseError(_) |
            SubscribeError::StoreTokenError(_) |
            SubscribeError::SendEmailError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            
        }
    }
}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A database error was encountered while saving token")
    }
}
fn chain_error(e: &impl std::error::Error, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;

    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by: \n\t{}", cause)?;
        current = cause.source();
    }

    Ok(())
}
impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        chain_error(&self.0, f)
    }
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
skip(new_subscriber, transaction),
fields(
subscriber_id = % new_subscriber.email.as_ref(),
subscriber_name = % new_subscriber.name.as_ref()
)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    transaction: &mut Transaction<'_, Postgres>,
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
    .execute(transaction)
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
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber: NewSubscriber = form.0.try_into()?;

    let mut transaction = pool.begin().await.unwrap();

    let subscriber_id = insert_subscriber(&new_subscriber, &mut transaction).await?;

    let subscription_token = generate_subscription_token();

    store_token(subscriber_id, &subscription_token, &mut transaction).await?;

    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await?;
    transaction.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
#[tracing::instrument(
    name = "saving token to database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    subscriber_id: Uuid,
    subscription_token: &str,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscriber_id, subscription_token) VALUES ($1,$2)"#,
        subscriber_id,
        subscription_token
    )
    .execute(transaction)
    .await
    .map_err(|e| StoreTokenError(e))?;

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
