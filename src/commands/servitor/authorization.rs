use serenity::all::{Role, User};
use crate::bot::{BotError, Context};
use super::autocomplete_server_name;

#[poise::command(slash_command, owners_only, rename = "add-user")]
pub async fn add_user(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	server: String,
	#[description = "User that be allowed operate this server"] user: User,
) -> Result<(), BotError> {
	todo!()
}

#[poise::command(slash_command, owners_only, rename = "remove-user")]
pub async fn remove_user(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	server: String,
	#[description = "User that will no longer be allowed operate this server"] user: User,
) -> Result<(), BotError> {
	todo!()
}

#[poise::command(slash_command, owners_only, rename = "add-role")]
pub async fn add_role(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	server: String,
	#[description = "Role that be allowed operate this server"] role: Role,
) -> Result<(), BotError> {
	todo!()
}

#[poise::command(slash_command, owners_only, rename = "remove-role")]
pub async fn remove_role(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	server: String,
	#[description = "Role that will no longer be allowed operate this server"] role: Role,
) -> Result<(), BotError> {
	todo!()
}
