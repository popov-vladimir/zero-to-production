#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configiration() -> Result<Settings, config::ConfigError> {
    // Ok(Settings(DatabaseSettings("","","",5432,"","")))
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("configuration"))?;

    settings.try_into()
}
