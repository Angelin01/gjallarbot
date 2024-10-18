#![feature(trait_alias)]

use anyhow::Result;
use log::{error, info};

mod services;
mod data;
mod commands;
mod bot;

#[tokio::main]
async fn main() -> Result<()> {
	env_logger::init();

	let token = match std::env::var("GJ_DISCORD_TOKEN") {
		Ok(token) => token,
		Err(e) => {
			error!("Please configure the GJ_DISCORD_TOKEN environment variable");
			return Err(anyhow::Error::from(e));
		}
	};

	info!("Starting Gjallarbot");

	bot::client(&token).await?.start().await?;

	Ok(())
}
