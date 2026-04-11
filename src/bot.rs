use crate::commands;
use crate::config::Config;
use crate::db::DbConnection;
use crate::services::servitor::HttpServitorController;
use anyhow::Result;
use log::{debug, error};
use poise::{serenity_prelude as serenity, Framework, FrameworkOptions};
use secrecy::ExposeSecret;
use serenity::Client;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct BotState<D: DbConnection> {
	pub conn: Arc<Mutex<D>>,
	pub servitor: Arc<BTreeMap<String, HttpServitorController>>,
}

pub type BotError = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a, D> = poise::Context<'a, BotState<D>, BotError>;

pub async fn client<D: DbConnection + 'static>(config: &Config, conn: D) -> Result<Client> {
	let intents = serenity::GatewayIntents::non_privileged();

	let client = serenity::ClientBuilder::new(config.bot.token.expose_secret(), intents)
		.framework(build_framework(&config, conn).await?)
		.await?;
	Ok(client)
}

async fn build_framework<D: DbConnection + 'static>(config: &Config, conn: D) -> Result<Framework<BotState<D>, BotError>> {
	let servitor_controllers = config
		.servitor
		.iter()
		.map(|(name, info)| {
			HttpServitorController::new(&info.url, info.token.as_ref())
				.map(|controller| (name.to_owned(), controller))
		})
		.collect::<Result<BTreeMap<_, _>, _>>()?;

	let servitor = Arc::new(servitor_controllers);
	let conn = Arc::new(Mutex::new(conn));
	Ok(Framework::builder()
		.options(framework_options())
		.setup(|ctx, _, framework| {
			Box::pin(async move {
				poise::builtins::register_globally(ctx, &framework.options().commands).await?;
				Ok(BotState {
					conn,
					servitor,
				})
			})
		})
		.build())
}

fn framework_options<D: DbConnection + 'static>() -> FrameworkOptions<BotState<D>, BotError> {
	FrameworkOptions {
		commands: commands::commands(),
		on_error: |error| Box::pin(on_error(error)),
		initialize_owners: true,
		reply_callback: Some(log_replies),
		..Default::default()
	}
}

fn log_replies<D: DbConnection>(_: Context<D>, reply: poise::CreateReply) -> poise::CreateReply {
	debug!("Replied with embeds {:?}", reply.embeds);
	reply
}

async fn on_error<D: DbConnection>(error: poise::FrameworkError<'_, BotState<D>, BotError>) {
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
