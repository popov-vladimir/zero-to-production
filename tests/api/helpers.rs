use sqlx::{PgConnection, Connection, Executor, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use once_cell::sync::Lazy;
use zero2prod::telemetry::{init_subscriber, get_subscriber};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer
}
impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        let response =  reqwest::Client::new()
            .post(format!("{}/subscriptions", &self.address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("failed to send request");
        response
    }
}
pub async fn configure_database(db_config: &DatabaseSettings) -> Result<(), std::io::Error> {
    let connection_strin_wo_db = db_config.connection_string_without_db();
    let mut connection = PgConnection::connect_with(&db_config.connect_without_db())
        .await
        .expect("failed to create connection");
    let query = format!(r#"create database "{}";"#, &db_config.database_name);
    println!("connection_string_wo_db {}", connection_strin_wo_db);
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

    Ok(())
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
    
    let email_server = MockServer::start().await;

    let config = {
        let mut c = get_configuration().expect("failed to get config");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };
    configure_database(&config.database).await.expect("failed to configure database");
    let application = Application::build(config.clone()).await.unwrap();
    let _ = tokio::spawn(application.server);
    
    TestApp {
        address: format!("http://127.0.0.1:{}", application.port),
        db_pool: get_connection_pool(config).await.unwrap(),
        email_server
    }
}
