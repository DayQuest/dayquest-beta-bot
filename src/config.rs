use figment::{
    providers::{Format, Json},
    Figment,
};
use log::info;
use poise::serenity_prelude::prelude::TypeMapKey;
use serde::{Deserialize, Serialize};

pub const BOT_TOKEN_KEY: &str = "TOKEN";
pub const JWT_TOKEN_KEY: &str = "JWT_TOKEN";
pub const ERROR_MSG: &str = "An error occurred. Please check logs!";
pub const NOT_PERMITTED: &str = "You are not permitted to do that!";
const FILE_PATH: &str = "config.json";

pub fn load() -> Config {
    let config = Figment::new().merge(Json::file(FILE_PATH));

    let config: Config = config.extract().expect("Failed to load config..");

    info!("Loaded config");
    config
}


impl TypeMapKey for Config {
    type Value = Config;
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub beta_addkey_url: String,
    pub beta_removekey_url: String,
    pub beta_getkey_url: String,
    pub beta_giver_role: u64,
}
