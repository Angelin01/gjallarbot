mod machine;

use poise::{serenity_prelude as serenity, CreateReply};
use crate::data::{Context, BotError};

#[poise::command(
	slash_command,
	rename="wake-on-lan",
	subcommands(
		"wake",
		"machine::add_machine",
		"machine::remove_machine",
		"machine::list_machines",
		"add_user",
		"remove_user",
		"add_role",
		"remove_role",
	),
	subcommand_required,
)]
pub async fn wake_on_lan(_: Context<'_>) -> Result<(), BotError> {
	panic!("Can't call parent commands");
}

#[poise::command(slash_command)]
async fn wake(
	ctx: Context<'_>,
	#[description = "Machine name"] name: String,
) -> Result<(), BotError> {
	ctx.send(CreateReply::default().ephemeral(true).content("It works")).await?;

	Ok(())
}

#[poise::command(slash_command, rename="add-user")]
async fn add_user(
	ctx: Context<'_>,
	#[description = "Machine name"] name: String,
	#[description = "User that will be allowed to turn this machine on"] user: serenity::User,
) -> Result<(), BotError> {
	ctx.send(CreateReply::default().ephemeral(true).content("It works")).await?;

	Ok(())
}

#[poise::command(slash_command, rename="remove-user")]
async fn remove_user(
	ctx: Context<'_>,
	#[description = "Machine name"] name: String,
	#[description = "User that will no longer be allowed to turn this machine on"] user: serenity::User,
) -> Result<(), BotError> {
	ctx.send(CreateReply::default().ephemeral(true).content("It works")).await?;

	Ok(())
}

#[poise::command(slash_command, rename="add-role")]
async fn add_role(
	ctx: Context<'_>,
	#[description = "Machine name"] name: String,
	#[description = "Role that will be allowed to turn this machine on"] role: serenity::Role,
) -> Result<(), BotError> {
	ctx.send(CreateReply::default().ephemeral(true).content("It works")).await?;

	Ok(())
}

#[poise::command(slash_command, rename="remove-role")]
async fn remove_role(
	ctx: Context<'_>,
	#[description = "Machine name"] name: String,
	#[description = "Role that will no longer be allowed to turn this machine on"] role: serenity::Role,
) -> Result<(), BotError> {
	ctx.send(CreateReply::default().ephemeral(true).content("It works")).await?;

	Ok(())
}
