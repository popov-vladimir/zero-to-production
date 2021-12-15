use sqlx::{PgConnection, Connection, Executor, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::startup::run;
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use once_cell::sync::Lazy;
use zero2prod::telemetry::{init_subscriber, get_subscriber};
use zero2prod::email_client::EmailClient;
use zero2prod::domain::SubscriberEmail;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
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
        init_subscriber(get_subscriber("test".into(), "info".into(), std::io::stdout))
    } else {
        init_subscriber(get_subscriber("test".into(), "info".into(), std::io::sink))
    }
});

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let database_name = Uuid::new_v4().to_string();

    let mut config = get_configuration().expect("failed to get config");
    config.database.database_name = database_name;
    let pool = configure_database(&config.database).await;
    let timeout = config.email_client.timeout();
    let server = run(listener, pool.clone(),
                     EmailClient::new(
                         config.email_client.base_url,
                         SubscriberEmail::parse(config.email_client.sender_email).unwrap(),
                         config.email_client.authorization_token,
                         timeout
                     )).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: pool,
    }
}
