use figment::{
    providers::{Format, Json},
    Figment,
};
use log::info;
use serde::{Deserialize, Serialize};
use serenity::prelude::TypeMapKey;

pub const TOKEN_KEY: &str = "TOKEN";
const FILE_PATH: &str = "config.json";

pub fn load() -> Config {
    let config = Figment::new().merge(Json::file(FILE_PATH));

    let config: Config = config.extract().expect("Failed to load config..");

    info!("Loaded config");
    config
}

pub struct ConfigData;

impl TypeMapKey for ConfigData {
    type Value = Config;
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub beta_addkey_url: String,
    pub beta_removekey_url: String,
    pub beta_getkey_url: String,
}
