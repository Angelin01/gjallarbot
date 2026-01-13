mod authorization;
mod machine;
mod wake;

use super::DISCORD_MAX_AUTOCOMPLETE_CHOICES;
use crate::bot::{BotError, Context};
use crate::db::DbConnection;

#[poise::command(
	slash_command,
	rename = "wake-on-lan",
	subcommands(
		"wake::wake",
		"machine::add_machine",
		"machine::remove_machine",
		"machine::list_machines",
		"machine::describe_machine",
		"authorization::add_user",
		"authorization::remove_user",
		"authorization::add_role",
		"authorization::remove_role",
	),
	subcommand_required
)]
pub async fn wake_on_lan<D: DbConnection>(_: Context<'_, D>) -> Result<(), BotError> {
	unreachable!("Can't call parent commands");
}

async fn autocomplete_machine_name<D: DbConnection>(ctx: Context<'_, D>, partial: &str) -> Vec<String> {
	ctx.data()
		.data
		.read()
		.await
		.wake_on_lan
		.keys()
		.filter(|name| name.starts_with(partial))
		.take(DISCORD_MAX_AUTOCOMPLETE_CHOICES)
		.cloned()
		.collect()
}
