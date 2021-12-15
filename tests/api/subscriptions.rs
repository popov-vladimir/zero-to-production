



use sqlx::PgPool;
use zero2prod::configuration::get_configuration;
use zero2prod::telemetry::{init_subscriber, get_subscriber};
use zero2prod::email_client::EmailClient;
use zero2prod::domain::SubscriberEmail;
use crate::helpers::spawn_app;

#[actix_rt::test]
async fn subscribe_returns_200_if_valid_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=test&email=test%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("select email, name from subscriptions;")
        .fetch_one(&app.db_pool)
        .await
        .expect("query failed");

    assert_eq!("test", saved.name);
    assert_eq!("test@gmail.com", saved.email);
}

#[actix_rt::test]
async fn subscribe_returns_400_if_missing_data() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let test_cases = vec![
        // ("email=test$%40gmail.com", "missing a name"),
        // ("name=name", "missing and email"),
        // ("", "missing both name and email"),
        // ("name=\\\\&email=test@test.com", "missing name"),
        ("name=valid_name&email=543", "bad email")
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("failed to send request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail when payload was {}",
            error_message
        )
    }
}
