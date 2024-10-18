use crate::data::{Context, BotError};

#[poise::command(slash_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), BotError> {
	poise::builtins::register_application_commands_buttons(ctx).await?;
	Ok(())
}
