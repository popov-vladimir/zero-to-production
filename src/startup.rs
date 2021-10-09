use sqlx::PgPool;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use std::net::TcpListener;

use crate::routes::*;

pub fn run(listener: TcpListener, pool: PgPool) -> Result<Server, std::io::Error> {

    let pool = web::Data::new(pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
