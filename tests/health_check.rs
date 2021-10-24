use sqlx::{PgConnection, Connection, Executor};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::startup::run;
use zero2prod::configuration::DatabaseSettings;

pub struct TestApp {
    address: String,
    db_pool: PgPool,
}

pub async fn configure_database(db_config: &DatabaseSettings) -> PgPool {
    let connection_strin_wo_db = db_config.connection_string_without_db();
    let mut connection = PgConnection::connect_with(&db_config.connect_without_db())
        .await
        .expect("failed to create connection");
    let query = format!(r#"create database "{}";"#, &db_config.database_name);
    println!("connection_strin_wo_db:q: {}", connection_strin_wo_db);
    println!("query: {}", query);
    connection.execute(&*query)
        .await
        .expect("failed to create database");

    let pool = PgPool::connect_with(db_config.with_db())
        .await
        .expect("failed to connect");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    pool
}

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
    init_subscriber(get_subscriber("test".into(), "info".into(),std::io::stdout))
    } else
    {
        init_subscriber(get_subscriber("test".into(), "info".into(),std::io::sink))
    }

});

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let database_name = Uuid::new_v4().to_string();

    let mut config = get_configuration().expect("failed to get config");
    config.database.database_name = database_name;
    let pool = configure_database(&config.database).await;
    let server = run(listener, pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: pool,
    }
}

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        // Use the returned application address
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

use sqlx::PgPool;
use zero2prod::configuration::get_configuration;
use zero2prod::telemetry::{init_subscriber, get_subscriber};
use once_cell::sync::Lazy;

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
        ("email=test$%40gmail.com", "missing a name"),
        ("name=name", "missing and email"),
        ("", "missing both name and email"),
        ("name=\\\\&email=test@test.com", "missing name")
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
