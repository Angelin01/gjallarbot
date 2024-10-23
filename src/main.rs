#![feature(trait_alias)]
#![feature(let_chains)]

use anyhow::Result;
use log::{error, info, LevelFilter};

mod services;
mod data;
mod commands;
mod bot;
mod errors;
mod embeds;

#[tokio::main]
async fn main() -> Result<()> {
	env_logger::builder()
		.filter_module("gjallarbot", LevelFilter::Info)
		.parse_env("GJ_LOG_LEVEL")
		.init();

	let token = match std::env::var("GJ_DISCORD_TOKEN") {
		Ok(token) => token,
		Err(e) => {
			error!("Please configure the GJ_DISCORD_TOKEN environment variable");
			return Err(anyhow::Error::from(e));
		}
	};

	info!("Starting Gjallarbot v{}", env!("CARGO_PKG_VERSION"));

	bot::client(&token).await?.start().await?;

	Ok(())
}
