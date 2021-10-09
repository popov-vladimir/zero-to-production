use sqlx::{PgPool, ConnectOptions};
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use env_logger::Env;
use sqlx::postgres::{PgConnectOptions};
use log::LevelFilter::{Off};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let configuration = get_configuration().expect("failed to get configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    let db = configuration.database;
    let pool = PgPool::connect_with(PgConnectOptions::new()
        .database(&db.database_name)
        .username(&db.username)
        .password(&db.password)
        .host(&db.host)
        .log_statements(Off)
        .to_owned()
    )
        .await
        .expect("failed to connect");

    run(listener,pool)?.await

}
