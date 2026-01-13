mod action;
mod authorization;
mod server;

use crate::bot::{BotError, Context};
use crate::commands::DISCORD_MAX_AUTOCOMPLETE_CHOICES;
use crate::db::DbConnection;

#[poise::command(
	slash_command,
	subcommands(
		"action::start",
		"action::stop",
		"action::restart",
		"action::reload",
		"action::status",
		"server::add_server",
		"server::remove_server",
		"server::list_servers",
		"server::describe_server",
		"authorization::add_user",
		"authorization::remove_user",
		"authorization::add_role",
		"authorization::remove_role",
	),
	subcommand_required
)]
pub async fn servitor<D: DbConnection>(_: Context<'_, D>) -> Result<(), BotError> {
	unreachable!("Can't call parent commands");
}

async fn autocomplete_server_name<D: DbConnection>(ctx: Context<'_, D>, partial: &str) -> Vec<String> {
	ctx.data()
		.data
		.read()
		.await
		.servitor
		.keys()
		.filter(|name| name.starts_with(partial))
		.take(DISCORD_MAX_AUTOCOMPLETE_CHOICES)
		.cloned()
		.collect()
}

async fn autocomplete_servitor_name<D: DbConnection>(ctx: Context<'_, D>, partial: &str) -> Vec<String> {
	ctx.data()
		.servitor
		.keys()
		.filter(|name| name.starts_with(partial))
		.take(DISCORD_MAX_AUTOCOMPLETE_CHOICES)
		.cloned()
		.collect()
}
