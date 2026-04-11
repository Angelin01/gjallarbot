use crate::bot::{BotError, Context};
use crate::db::DbConnection;

/// Debugging: register commands
#[poise::command(slash_command, owners_only)]
pub async fn register<D: DbConnection>(ctx: Context<'_, D>) -> Result<(), BotError> {
	poise::builtins::register_application_commands_buttons(ctx).await?;
	Ok(())
}
