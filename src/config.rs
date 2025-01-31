use figment::{providers::{Format, Json}, Figment};
use log::info;
use serde::{Deserialize, Serialize};

pub const TOKEN_KEY: &str = "TOKEN";
const FILE_PATH: &str = "config.json";

pub fn load() -> Config {
    let config = Figment::new().merge(Json::file(FILE_PATH));

    let config: Config = config.extract().expect("Failed to load config..");

    info!("Loaded config");
    config
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub beta_add_url: String,
}