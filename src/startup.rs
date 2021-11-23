use sqlx::PgPool;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use crate::routes::*;
use crate::email_client::EmailClient;

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
