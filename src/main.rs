use sqlx::{PgPool, ConnectOptions};
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::*;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use log::LevelFilter::{Off};
use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    init_subscriber(get_subscriber("zero2prod".into(), "trace".into(), std::io::stdout));

    let configuration = get_configuration().expect("failed to get configuration");
    let address = format!("127.0.0.1:{}", configuration.application.port);
    let listener = TcpListener::bind(address)?;

    let db = configuration.database;

    let pool = PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect_with(PgConnectOptions::new()
            .database(&db.database_name)
            .username(&db.username)
            .password(&db.password)
            .host(&db.host)
            .log_statements(Off)
            .to_owned()).await
        .expect("failed to connect");


    run(listener, pool)?.await
}
