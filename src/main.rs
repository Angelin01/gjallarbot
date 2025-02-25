#![feature(trait_alias)]
#![feature(let_chains)]
#![feature(async_closure)]
#![feature(async_fn_traits)]

use crate::config::{Config, LogConfig};
use anyhow::Result;
use log::info;
use serenity::all::ShardManager;
use std::sync::Arc;
use tokio::signal;
use tracing_subscriber::filter::LevelFilter;

mod bot;
mod commands;
mod config;
mod controllers;
mod data;
mod embeds;
mod errors;
mod services;
mod views;

#[tokio::main]
async fn main() -> Result<()> {
	let config = Config::load()?;

	setup_logging(&config.log);

	let config = Config::load()?;

	let mut bot = bot::client(&config).await?;

	drop(config);

	tokio::spawn(graceful_shutdown(bot.shard_manager.clone()));

	info!("Starting Gjallarbot v{}", env!("CARGO_PKG_VERSION"));

	bot.start().await?;

	Ok(())
}

fn setup_logging(log_config: &LogConfig) {
	tracing_subscriber::fmt()
		.with_max_level(LevelFilter::INFO)
		.with_env_filter(&log_config.filter)
		.init();
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
