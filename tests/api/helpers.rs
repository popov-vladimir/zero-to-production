use once_cell::sync::Lazy;
use reqwest::Url;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub port: u16,
    pub email_server: MockServer,
}
pub struct ConfirmationLink {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}
impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        let response = reqwest::Client::new()
            .post(format!("{}/subscriptions", &self.address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("failed to send request");
        response
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLink {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();

            assert_eq!(links.len(), 1);

            let mut confirmation_link = Url::parse(links.get(0).unwrap().as_str()).unwrap();

            confirmation_link.set_port(Some(self.port)).unwrap();
            return confirmation_link;
        };

        ConfirmationLink {
            html: get_link(body["HtmlBody"].as_str().unwrap()),
            plain_text: (get_link(body["TextBody"].as_str().unwrap())),
        }
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
    connection
        .execute(&*query)
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
        init_subscriber(get_subscriber(
            "test".into(),
            "info".into(),
            std::io::stdout,
        ))
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
    configure_database(&config.database)
        .await
        .expect("failed to configure database");
    let application = Application::build(config.clone()).await.unwrap();

    let address = format!("http://127.0.0.1:{}", application.port());

    let port = application.port();

    let _ = tokio::spawn(application.run_untill_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(config).await.unwrap(),
        port,
        email_server,
    }
}
