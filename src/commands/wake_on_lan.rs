mod authorization;
mod machine;
mod wake;

use super::DISCORD_MAX_AUTOCOMPLETE_CHOICES;
use crate::bot::{BotError, Context};
use crate::db::DbConnection;
use crate::schema::wake_on_lan_machines;
use diesel::{QueryDsl, TextExpressionMethods};
use diesel_async::RunQueryDsl;

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
	let mut conn = ctx.data().conn.lock().await;
	wake_on_lan_machines::table
		.select(wake_on_lan_machines::name)
		.filter(wake_on_lan_machines::name.like(format!("{partial}%")))
		.limit(DISCORD_MAX_AUTOCOMPLETE_CHOICES)
		.load::<String>(&mut *conn)
		.await
		.unwrap_or_default()
}
