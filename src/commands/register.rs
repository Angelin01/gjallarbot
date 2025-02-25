use crate::bot::{BotError, Context};
/// Debugging: register commands
#[poise::command(slash_command, owners_only)]
pub async fn register(ctx: Context<'_>) -> Result<(), BotError> {
	poise::builtins::register_application_commands_buttons(ctx).await?;
	Ok(())
}
