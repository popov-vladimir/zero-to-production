use std::result::Result;
use std::time::Duration;
use sqlx::PgPool;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use crate::configuration::Settings;
use crate::domain::SubscriberEmail;
use crate::routes::*;
use crate::email_client::EmailClient;


pub struct Application {
    pub server: Server,
    pub port: u16
}

impl Application {
    pub async fn build(configuration:Settings)-> Result<Self, std::io::Error> {
        let address = format!("{}:{}", configuration.application.host, configuration.application.port);
        let listener = TcpListener::bind(address)?;
        let port = listener
        .local_addr()
        .unwrap()
        .port();
    
        let pool = PgPoolOptions::new()
            .connect_timeout(Duration::from_secs(2))
            .connect_with(configuration.database.with_db())
            .await
            .expect("failed to connect");
    
        tracing::debug!("connection to db was successful");
    
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            SubscriberEmail::parse(configuration.email_client.sender_email).unwrap(),
            configuration.email_client.authorization_token,
            timeout
        );
        
        Ok(
            Self{ server: run(listener, pool, email_client)?, port }
        )
    }
    
}
pub fn run(listener: TcpListener, pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {

    let pool = web::Data::new(pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {

        tracing::debug!("starting the webserver");
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub async fn get_connection_pool(configuration:Settings) -> Result<PgPool, sqlx::Error>
{
    PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect_with(configuration.database.with_db())
        .await

}
pub async fn build(configuration:Settings)-> Result<Server, std::io::Error> {
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;

    let pool = PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect_with(configuration.database.with_db())
        .await
        .expect("failed to connect");

    tracing::debug!("connection to db was successful");

    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        SubscriberEmail::parse(configuration.email_client.sender_email).unwrap(),
        configuration.email_client.authorization_token,
        timeout
    );
    run(listener, pool, email_client)
}
