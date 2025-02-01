
use log::{error, info, warn};
use poise::serenity_prelude::{CreateMessage, Mentionable, User};
use serde::{Deserialize, Serialize};
use crate::{config::{Config, ERROR_MSG, NOT_PERMITTED}, Data};

type Error = Box<dyn std::error::Error + Send + Sync>;

//Same content
type GetResponse = AddResponse;

#[derive(Deserialize, Serialize, Clone)]
struct AddResponse {
    key: String
}


#[derive(Deserialize, Serialize, Clone)]
struct RemoveResponse {}

#[poise::command(
    slash_command,
    subcommands("add", "remove", "get"),
    guild_only
)]
pub async fn beta(ctx: crate::Context<'_>) -> Result<(), Error> {
    ctx.say("Please specify a subcommand: add, remove or get.")
        .await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn add(
    ctx: crate::Context<'_>,
    #[description = "The user to add"] user: User,
) -> Result<(), Error> {
    let data = ctx.data();

    if !is_permitted(&user, ctx, &data.config).await {
        ctx.defer_ephemeral().await.ok();
        ctx.say(NOT_PERMITTED).await.ok();
        return Ok(());
    }

    match send_request::<AddResponse>(user.id.get(), &data.config.beta_addkey_url, data).await {
        Ok(reponse) => {
            info!("Added beta user: {}({})", user.name, user.id);

            if let Err(why) = user.dm(ctx.http(), CreateMessage::new()
            .content(format!("✅ Du wurdest ins Betaprogramm aufgenommen! Bitte teile deinen Betakey niemals mit dritten! Dein Betakey: ||{}||", reponse.key)))
            .await {
                warn!("Unable to send beta key dm to user {} ({}): {}", user.name, user.id.get(), why);
                ctx.reply("Unable to send beta dm (dms deactivated?)").await.ok();
            }
            ctx.reply(format!(
                "User {} added to the beta program.",
                user.mention()
            ))
            .await?;
        },
        Err(why) => {
            ctx.reply(ERROR_MSG).await.ok();
            error!("Failed adding beta user: {}", why);
        },
    }
   
    Ok(())
}

#[poise::command(slash_command)]
pub async fn remove(
    ctx: crate::Context<'_>,
    #[description = "The user to remove"] user: User,
) -> Result<(), Error> {
    let data = ctx.data();
    if !is_permitted(&user, ctx, &data.config).await {
        ctx.defer_ephemeral().await.ok();
        ctx.say(NOT_PERMITTED).await.ok();
        return Ok(());
    }
    match send_request::<RemoveResponse>(user.id.get(), &data.config.beta_removekey_url, data).await {
        Ok(_) => {
            info!("Removed beta user: {}({})", user.name, user.id.get());
            ctx.reply(format!(
                "User {} removed from beta program.",
                user.mention()
            ))
            .await?;
        },
        Err(why) => {
            ctx.reply(ERROR_MSG).await.ok();
            error!("Failed removing beta user: {}", why);
        },
    }
   
    Ok(())
}

#[poise::command(slash_command)]
pub async fn get(
    ctx: crate::Context<'_>,
    #[description = "The user to fetch the key from"] user: User,
) -> Result<(), Error> {
    let data = ctx.data();
    if !is_permitted(&user, ctx, &data.config).await {
        ctx.defer_ephemeral().await.ok();
        ctx.say(NOT_PERMITTED).await.ok();
        return Ok(());
    }
    match send_request::<GetResponse>(user.id.get(), &data.config.beta_getkey_url, data).await {
        Ok(reponse) => {
            info!("Fetched key: {} of beta user: {}({})", reponse.key, user.name, user.id);
            ctx.defer_ephemeral().await.ok();
            ctx.say(format!("{}'s beta key is: ||{}||", user.mention(), reponse.key)).await.ok();
        
        },
        Err(why) => {
            ctx.reply(ERROR_MSG).await.ok();
            error!("Failed fetching user's key: {}", why);
        },
    }
   
    Ok(())
}



async fn send_request<T>(discord_id: u64, url: &String, data: &Data) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    match data
        .reqwest
        .post(url)
        .bearer_auth(data.jwt.clone())
        .json(&serde_json::json!({
            "discord_id": discord_id
        }))
        .send()
        .await
    {
        Ok(response) => {
            if !response.status().is_success() {
                return Err(format!("Got status code: {}", response.status()));
            }

            response
                .json::<T>()
                .await
                .map_err(|e| format!("Failed to deserialize response: {}", e))
        }

        Err(why) => {
            error!("Error while sending request: {}", why);
            Err(format!("Request failed: {}", why))
        }
    }
}

async fn is_permitted(user: &User, ctx: crate::Context<'_>, config: &Config) -> bool {
    user.has_role(ctx.http(), ctx.guild_id().unwrap().get(), config.beta_giver_role).await.ok().unwrap()
}