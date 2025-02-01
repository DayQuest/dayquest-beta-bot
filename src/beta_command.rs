use crate::{
    config::{Config, ERROR_MSG, NOT_PERMITTED},
    Data,
};
use log::{error, info, warn};
use poise::serenity_prelude::{CreateMessage, Mentionable, User};
use reqwest::{Response, StatusCode};
type Error = Box<dyn std::error::Error + Send + Sync>;

#[poise::command(slash_command, subcommands("add", "remove", "get"), guild_only)]
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

    if !is_permitted(&ctx.author(), ctx, &data.config).await {
        ctx.defer_ephemeral().await.ok();
        ctx.say(NOT_PERMITTED).await.ok();
        return Ok(());
    }

    if let Some(res) = send_request(user.id.get(), &data.config.beta_addkey_url, data, ctx).await {

        if res.status() == StatusCode::UNPROCESSABLE_ENTITY {
            ctx.defer_ephemeral().await?;
            ctx.say(format!("❌: {} is in the beta program already", user.mention())).await?;
            return Ok(());
        }


        if res.status() == StatusCode::OK {
            let key = res.text().await.unwrap();
            info!("Added beta user: {}({}), key: {}", user.name, user.id, key);
    
            if let Err(why) = user.dm(ctx.http(), CreateMessage::new()
            .content(format!("✅ Du wurdest ins Betaprogramm aufgenommen!\n Bitte teile deinen Betakey **niemals** mit dritten!\n Dein Betakey: ||{}||", key)))
            .await {
                warn!("Unable to send beta key dm to user {} ({}): {}", user.name, user.id.get(), why);
                ctx.reply("Unable to send beta dm (dms deactivated?)").await.ok();
            }
            ctx.reply(format!(
                "✅: User {} added to the beta program.",
                user.mention()
            ))
            .await?;
            if let Err(why) = change_beta_role(ctx, &user, true, data.config.beta_role).await {
                warn!("Failed to update beta role: {}", why);
            }
        } else {
            print_unexpected_status(ctx, res).await?;
            return Ok(());
        }
   
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn remove(
    ctx: crate::Context<'_>,
    #[description = "The user to remove"] user: User,
) -> Result<(), Error> {
    let data = ctx.data();
    if !is_permitted(&ctx.author(), ctx, &data.config).await {
        ctx.defer_ephemeral().await?;
        ctx.say(NOT_PERMITTED).await?;
        return Ok(());
    }

    if let Some(res) = send_request(user.id.get(), &data.config.beta_removekey_url, data, ctx).await {
        if res.status() == StatusCode::UNPROCESSABLE_ENTITY {
            ctx.defer_ephemeral().await?;
            ctx.say(format!("❌: {} is not in the beta program", user.mention())).await?;
            return Ok(());
        }

        if res.status() == StatusCode::OK {
            info!("Removed beta user: {}({})", user.name, user.id.get());
            ctx.reply(format!(
                "✅: User {} removed from beta program.",
                user.mention()
            ))
            .await?;

            if let Err(why) = change_beta_role(ctx, &user, false, data.config.beta_role).await {
                warn!("Failed to update beta role: {}", why);
            } else {
                print_unexpected_status(ctx, res).await?;
                return Ok(());
            }

            return Ok(());
        }
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn get(
    ctx: crate::Context<'_>,
    #[description = "The user to fetch the key from"] user: User,
) -> Result<(), Error> {
    let data = ctx.data();
    if !is_permitted(&ctx.author(), ctx, &data.config).await {
        ctx.defer_ephemeral().await?;
        ctx.say(NOT_PERMITTED).await?;
        return Ok(());
    }
    ctx.defer_ephemeral().await?;
    if let Some(res) = send_request(user.id.get(), &data.config.beta_getkey_url, data, ctx).await {
        // later change to 404 when changed in api
        if res.status() == StatusCode::UNPROCESSABLE_ENTITY {
            ctx.defer_ephemeral().await?;
            ctx.say(format!("❌: {} is not in the beta program", user.mention()))
                .await?;
            return Ok(());
        }

        if res.status() == StatusCode::OK {
            let key = res.text().await.unwrap();
            info!(
                "Fetched key: {} of beta user: {}({})",
                key, user.name, user.id
            );
            ctx.defer_ephemeral().await.ok();
            ctx.say(format!("✅: {}'s beta key is: ||{}||", user.mention(), key))
                .await?;
            return Ok(());
        } else {
            print_unexpected_status(ctx, res).await?;
            return Ok(());
        }
    }

    Ok(())
}


async fn print_unexpected_status(ctx: crate::Context<'_>, res: Response) -> Result<(), Error> {
    ctx.say(format!(
        "{ERROR_MSG}\nError (Unexpected StatusCode): ```{:?}```",
        res.error_for_status()
    ))
    .await?;

    Ok(())
}

async fn send_request(
    discord_id: u64,
    url: &String,
    data: &Data,
    ctx: crate::Context<'_>,
) -> Option<Response> {
    match data
        .reqwest
        .post(url)
        .bearer_auth(data.jwt.clone())
        .json(&serde_json::json!({ "discordId": discord_id }))
        .send()
        .await
    {
        Ok(response) => Some(response),
        Err(why) => {
            ctx.reply(ERROR_MSG).await.ok();
            error!("Failed while sending request: {}", why);
            None
        }
    }
}

async fn is_permitted(user: &User, ctx: crate::Context<'_>, config: &Config) -> bool {
    user.has_role(
        ctx.http(),
        ctx.guild_id().unwrap().get(),
        config.beta_giver_role,
    )
    .await
    .ok()
    .unwrap()
}

async fn change_beta_role(
    ctx: crate::Context<'_>,
    user: &User,
    add: bool,
    beta_role: u64,
) -> Result<(), Error> {
    let member = ctx
        .guild_id()
        .unwrap()
        .member(ctx.http(), user.id)
        .await
        .unwrap();
    if add {
        member.add_role(ctx.http(), beta_role).await?;
    } else {
        member.remove_role(ctx.http(), beta_role).await?;
    }

    Ok(())
}
