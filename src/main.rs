use std::{env, process::exit, time::Instant};

use colored::Colorize;
use config::TOKEN_KEY;
use env_logger::{Builder, Env};
use log::{error, info};
use serenity::{all::GatewayIntents, client, Client, Error};
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
    let mut client = Client::builder(&token, GatewayIntents::all()).await?;

    let config = config::load();
    info!("Bot running, took: {} ms", start_time.elapsed().as_millis());
    
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
