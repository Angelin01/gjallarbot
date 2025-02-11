#![feature(trait_alias)]
#![feature(let_chains)]

use std::sync::Arc;
use anyhow::Result;
use log::{error, info, LevelFilter};
use serenity::all::ShardManager;
use tokio::signal;

mod services;
mod data;
mod commands;
mod bot;
mod errors;
mod embeds;
mod controllers;

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

	let mut bot = bot::client(&token).await?;

	tokio::spawn(graceful_shutdown(bot.shard_manager.clone()));

	info!("Starting Gjallarbot v{}", env!("CARGO_PKG_VERSION"));

	bot.start().await?;

	Ok(())
}

async fn graceful_shutdown(shard_manager: Arc<ShardManager>) {
	let ctrl_c = async {
		signal::ctrl_c()
			.await
			.expect("failed to install interrupt handler");
	};

	#[cfg(unix)]
	let terminate = async {
		signal::unix::signal(signal::unix::SignalKind::terminate())
			.expect("failed to install SIGTERM handler")
			.recv()
			.await;
	};

	#[cfg(not(unix))]
	let terminate = std::future::pending::<()>();

	let received_shutdown = tokio::select! {
		biased;
		_ = ctrl_c => true,
		_ = terminate => true,
		else => false
	};

	if received_shutdown {
		info!("Received signal, shutting down");
		shard_manager.shutdown_all().await;
	}
}
