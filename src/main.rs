use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::*;
use sqlx::postgres::{PgPoolOptions};
use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    init_subscriber(get_subscriber("zero2prod".into(), "trace".into(), std::io::stdout));

    let configuration = get_configuration().expect("failed to get configuration");
    let address = format!("127.0.0.1:{}", configuration.application.port);
    let listener = TcpListener::bind(address)?;

    let pool = PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect_with(configuration.database.with_db())
        .await
        .expect("failed to connect");

    tracing::debug!("connection to db was successful");

    run(listener, pool)?.await
}
