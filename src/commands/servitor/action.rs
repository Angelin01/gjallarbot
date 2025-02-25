use super::autocomplete_server_name;
use crate::bot::{BotError, Context};

#[poise::command(slash_command)]
pub async fn start(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	todo!()
}

#[poise::command(slash_command)]
pub async fn stop(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	todo!()
}

#[poise::command(slash_command)]
pub async fn restart(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	todo!()
}

#[poise::command(slash_command)]
pub async fn reload(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	todo!()
}

#[poise::command(slash_command)]
pub async fn status(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	todo!()
}
