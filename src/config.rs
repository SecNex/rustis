use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DbSettings,
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub address: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct DbSettings {
    pub host : Option<String>,
    pub port : Option<u16>,
    pub user : Option<String>,
    pub password : Option<String>,
    pub dbname : Option<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let builder = Config::builder()
            .add_source(File::new("config/settings.conf", FileFormat::Toml));
        let settings = builder.build()?;
        settings.try_deserialize()
    }
}