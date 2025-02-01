use std::{env, process::exit, sync::Arc, time::Instant};

use colored::Colorize;
use config::{Config, ConfigData, TOKEN_KEY};
use env_logger::{Builder, Env};
use log::{error, info};
use serenity::{all::{EventHandler, GatewayIntents}, prelude::TypeMapKey, Client, Error};
mod beta_command;
mod config;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let start_time = Instant::now();
    ctrlc::set_handler(move || {
        info!("{}", "Stopping bot, Bye :)".on_red());
        exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let dotenv_res = dotenv::dotenv();

    setup_logging();
    info!("Starting..");
    if let Ok(_) = dotenv_res {
        info!("Loaded .env file {}", "(development only)".yellow())
    }

    let token = env::var(TOKEN_KEY).expect("Unable to find token in enviroment");
    let mut client = Client::builder(&token, GatewayIntents::all())
        .event_handler(beta_command::Handler)
        .await?;

    let config = config::load();

    {
        let mut data = client.data.write().await;
        data.insert::<ConfigData>(config);
    }

  

    if let Err(why) = client.start().await {
        error!("Failed to start bot: {}", why);
    }
    Ok(())
}


fn setup_logging() {
    Builder::from_env(Env::default())
        .format_target(false)
        .init();
}
