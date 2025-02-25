use super::autocomplete_machine_name;
use crate::commands::reply_no_mentions;
use crate::{controllers, views};
use controllers::wake_on_lan::machine as ctrl_wol_mch;
use views::wake_on_lan::machine as view_wol_mch;
use crate::bot::{BotError, Context};

#[poise::command(slash_command, owners_only, rename = "add-machine")]
pub async fn add_machine(
	ctx: Context<'_>,
	#[description = "Machine name"] name: String,
	#[description = "Machine MAC Address as hex digits separated by :"] mac: String,
) -> Result<(), BotError> {
	let result = ctrl_wol_mch::add_machine(&ctx.data().data, &name, &mac).await;
	let embed = view_wol_mch::add_machine_embed(result, &name, &mac);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command, owners_only, rename = "remove-machine")]
pub async fn remove_machine(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	name: String,
) -> Result<(), BotError> {
	let result = ctrl_wol_mch::remove_machine(&ctx.data().data, &name).await;
	let embed = view_wol_mch::remove_machine_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command, rename = "list-machines")]
pub async fn list_machines(ctx: Context<'_>) -> Result<(), BotError> {
	let embed = ctrl_wol_mch::list_machines(&ctx.data().data, async |info| {
		view_wol_mch::list_machines_embed(info)
	})
	.await;

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command, rename = "describe-machine")]
pub async fn describe_machine(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	name: String,
) -> Result<(), BotError> {
	let embed = ctrl_wol_mch::describe_machine(&ctx.data().data, &name, async |result, name| {
		view_wol_mch::describe_machine_embed(result, &name)
	})
	.await;

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}
