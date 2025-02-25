use super::{autocomplete_server_name, autocomplete_servitor_name};
use crate::bot::{BotError, Context};

#[poise::command(slash_command, owners_only, rename = "add-server")]
pub async fn add_server(
	ctx: Context<'_>,
	#[description = "Server name"] name: String,
	#[description = "Servitor instance"]
	#[autocomplete = "autocomplete_servitor_name"]
	servitor: String,
	#[description = "Unit name"] unit_name: String,
) -> Result<(), BotError> {
	todo!()
}

#[poise::command(slash_command, owners_only, rename = "remove-server")]
pub async fn remove_server(
	ctx: Context<'_>,
	#[description = "Server name"] name: String,
) -> Result<(), BotError> {
	todo!()
}

#[poise::command(slash_command, rename = "list-servers")]
pub async fn list_servers(
	ctx: Context<'_>,
	#[description = "Servitor name to filter by"]
	#[autocomplete = "autocomplete_servitor_name"]
	servitor: Option<String>,
) -> Result<(), BotError> {
	todo!()
}

#[poise::command(slash_command, rename = "describe-server")]
pub async fn describe_server(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	todo!()
}
