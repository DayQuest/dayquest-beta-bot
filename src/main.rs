use std::{env, process::exit};

use colored::Colorize;
use config::{Config, BOT_TOKEN_KEY, JWT_TOKEN_KEY};
use env_logger::{Builder, Env};
use log::info;
use poise::{
    serenity_prelude::{
        self, async_trait, ClientBuilder, EventHandler, GatewayIntents, OnlineStatus, Ready
    },
    Framework, FrameworkOptions,
};
use reqwest::Client;
mod beta_command;
mod config;

pub struct Data {
    config: Config,
    reqwest: Client,
    jwt: String,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
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
    
    let config = config::load();
    let token = env::var(BOT_TOKEN_KEY).expect("Unable to find token in enviroment");

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![
                beta_command::beta(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, config.guild_id.into()).await?;

                info!("Registered commands globally");
                Ok(Data {
                    config, 
                    reqwest: Client::new(),
                    jwt: env::var(JWT_TOKEN_KEY).expect("Failed to get backend jwt from env"),
                })
            })
        })
        .build();

    let mut client = ClientBuilder::new(token, GatewayIntents::all())
        .framework(framework)
        .event_handler(Handler)
        .await
        .unwrap();

    client.start().await.unwrap();
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: serenity_prelude::Context, ready: Ready) {
        info!("{} is connected", ready.user.name);
        ctx.set_presence(None, OnlineStatus::Online);
    }
}

fn setup_logging() {
    Builder::from_env(Env::default())
        .format_target(false)
        .init();
}
