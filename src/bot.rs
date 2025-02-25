use crate::commands;
use crate::config::Config;
use crate::data::{BotData, PersistentJson};
use crate::services::servitor::{HttpServitorController};
use anyhow::Result;
use log::{debug, error};
use poise::{serenity_prelude as serenity, Framework, FrameworkOptions};
use secrecy::ExposeSecret;
use serenity::{Client};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct BotState {
	pub data: BotData,
	pub servitor: Arc<BTreeMap<String, HttpServitorController>>,
}

pub type BotError = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, BotState, BotError>;

pub async fn client(config: &Config) -> Result<Client> {
	let intents = serenity::GatewayIntents::non_privileged();

	let client = serenity::ClientBuilder::new(config.bot.token.expose_secret(), intents)
		.framework(build_framework().await)
		.await?;
	Ok(client)
}

async fn build_framework() -> Framework<BotState, BotError> {
	Framework::builder()
		.options(framework_options())
		.setup(|ctx, _, framework| {
			Box::pin(async move {
				poise::builtins::register_globally(ctx, &framework.options().commands).await?;
				Ok(BotState {
					data: Arc::new(RwLock::new(PersistentJson::new("data.json")?)),
					servitor: Arc::new(BTreeMap::new()),
				})
			})
		})
		.build()
}

fn framework_options() -> FrameworkOptions<BotState, BotError> {
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

async fn on_error(error: poise::FrameworkError<'_, BotState, BotError>) {
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
