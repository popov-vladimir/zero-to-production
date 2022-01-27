// use config::Environment;
use log::LevelFilter::Off;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;
use std::convert::{TryFrom, TryInto};
use std::time::Duration;
use tracing::log;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: String,
    timeout: u64,
}

impl EmailClientSettings {
    pub fn timeout(&self) -> std::time::Duration {
        Duration::from_millis(self.timeout)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(serde::Deserialize, std::fmt::Debug, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Ok(Settings(DatabaseSettings("","","",5432,"","")))

    let mut settings = config::Config::default();
    let base_path = std::env::current_dir().expect("failed to determine current dir");
    let configuration_directory = base_path.join("configuration");

    settings.merge(config::File::from(configuration_directory.join("base.yaml")).required(true))?;

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("failed to parse environment");
    tracing::debug!("the environment is {}", environment.as_str());

    settings
        .merge(
            config::File::from(configuration_directory.join(environment.as_str())).required(true),
        )
        .expect("failed to apply env settings");

    settings.merge(config::Environment::with_prefix("app").separator("__"))?;
    settings.try_into()
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        let result = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        );
        tracing::debug!("connection string is {}", result);
        result
    }
    pub fn connect_without_db(&self) -> PgConnectOptions {
        tracing::debug!("db settings {:?}", self);
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .password(&self.password)
            .username(&self.username)
            .ssl_mode(ssl_mode)
            .log_statements(Off)
            .to_owned()
    }
    pub fn with_db(&self) -> PgConnectOptions {
        self.connect_without_db().database(&self.database_name)
    }
    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local.yaml",
            Environment::Production => "production.yaml",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            _other => Err(format!("failed to parse {}", s)),
        }
    }
}
