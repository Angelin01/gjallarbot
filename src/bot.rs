use std::sync::Arc;
use anyhow::Result;
use log::{debug, error, warn};
use poise::{serenity_prelude as serenity, BoxFuture, Framework, FrameworkOptions};
use serenity::{Client, Ready};
use tokio::sync::RwLock;
use crate::commands;
use crate::data::{Context, BotData, BotError, PersistentJson};

pub async fn client(token: impl AsRef<str>) -> Result<Client> {
	let intents = serenity::GatewayIntents::non_privileged();

	let client = serenity::ClientBuilder::new(token, intents)
		.framework(build_framework().await)
		.await?;
	Ok(client)
}

async fn build_framework() -> Framework<BotData, BotError> {
	Framework::builder()
		.options(framework_options())
		.setup(setup)
		.build()
}

fn framework_options() -> FrameworkOptions<BotData, BotError> {
	FrameworkOptions {
		commands: commands::commands(),
		on_error: |error| Box::pin(on_error(error)),
		initialize_owners: true,
		reply_callback: Some(log_replies),
		..Default::default()
	}
}

fn log_replies(_: Context, reply: poise::CreateReply) -> poise::CreateReply {
	debug!("Replied with embeds {:?}", reply.embeds);
	reply
}

fn setup<'a>(ctx: &'a serenity::Context, _: &'a Ready, framework: &'a Framework<BotData, BotError>) -> BoxFuture<'a, serenity::Result<BotData, BotError>> {
	Box::pin(async move {
		if cfg!(debug_assertions) {
			if let (Ok(token), Ok(app_id), Ok(guild_id)) = (
				std::env::var("GJ_DISCORD_TOKEN"),
				std::env::var("GJ_APPLICATION_ID").and_then(|id| id.parse::<u64>().map_err(|_| std::env::VarError::NotPresent)),
				std::env::var("GJ_GUILD_ID").and_then(|id| id.parse::<u64>().map_err(|_| std::env::VarError::NotPresent)),
			) {
				let http_client = serenity::http::Http::new(&token);
				http_client.set_application_id(serenity::ApplicationId::new(app_id));

				if let Err(err) = poise::builtins::register_in_guild(
					&http_client,
					&framework.options().commands,
					serenity::GuildId::new(guild_id),
				).await {
					warn!("Failed to register commands in guild: {:?}", err);
				}
			} else {
				warn!(
					"In debug mode, but variables GJ_APPLICATION_ID and GJ_GUILD_ID are not set. \
					 Configure these to quickly register commands in a guild for debugging."
				);
			}
		}

		poise::builtins::register_globally(ctx, &framework.options().commands).await?;
		Ok(Arc::new(RwLock::new(PersistentJson::new("data.json")?)))
	})
}

async fn on_error(error: poise::FrameworkError<'_, BotData, BotError>) {
	match error {
		poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
		poise::FrameworkError::Command { error, ctx, .. } => {
			error!("Error in command `{}`: {:?}", ctx.command().name, error,);
		}
		error => {
			if let Err(e) = poise::builtins::on_error(error).await {
				error!("Error while handling error: {}", e)
			}
		}
	}
}
