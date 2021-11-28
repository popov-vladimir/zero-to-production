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
skip(new_subscriber, pool),
fields(
subscriber_id = % new_subscriber.email.as_ref(),
subscriber_name = % new_subscriber.name.as_ref()
)
)]
pub async fn insert_subscriber(new_subscriber: &NewSubscriber, pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into subscriptions(id, email, name, subscribed_at) values($1,$2,$3,$4)"#,
        Uuid::new_v4(),
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
    Ok(())
}

use crate::domain::{NewSubscriber, SubscriberName, SubscriberEmail};

use std::convert::{TryFrom,TryInto};

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name =  SubscriberName::parse(form.name)?;

        let email = SubscriberEmail::parse(form.email)?;

        Ok(NewSubscriber{name,email})
    }
}

pub fn parse_subscriber(form: web::Form<FormData>) -> Result<NewSubscriber,String> {

    let name =  SubscriberName::parse(form.0.name)?;

    let email = SubscriberEmail::parse(form.0.email)?;

    Ok(NewSubscriber{name,email})
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
    // Uuid::new_v4().to_string();
    //
    // let name = match SubscriberName::parse(form.0.name)
    //  {
    //     Ok(name) => name,
    //     Err(_) => return HttpResponse::BadRequest().finish()
    // };
    // let email = match SubscriberEmail::parse(form.0.email)
    //  {
    //     Ok(email) => email,
    //     Err(_) => return HttpResponse::BadRequest().finish()
    // };

    // let new_subscriber: NewSubscriber = match parse_subscriber(form)  {
    //     Ok(v) => v,
    //     Err(_) => return HttpResponse::BadRequest().finish()
    // };

    let new_subscriber:NewSubscriber = match form.0.try_into() {
      Ok(new_subscriber) => new_subscriber,
        Err(_) => return HttpResponse::BadRequest().finish()
    };


    match insert_subscriber(&new_subscriber, &pool).await
    {
        Ok(_) => {
            HttpResponse::Ok().finish()
        }
        Err(_) => {
            HttpResponse::InternalServerError().finish()
        }
    }
}
