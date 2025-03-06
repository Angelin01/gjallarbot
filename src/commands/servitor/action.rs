use super::super::reply_no_mentions;
use super::autocomplete_server_name;
use crate::bot::{BotError, Context};

use crate::controllers::servitor::action as ctrl_serv_act;
use crate::views::servitor::action as view_serv_act;

#[poise::command(slash_command)]
pub async fn start(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	let result =
		ctrl_serv_act::start(&ctx.data().data, &ctx.data().servitor, &name, ctx.author()).await;

	let embed = view_serv_act::start_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command)]
pub async fn stop(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	let result =
		ctrl_serv_act::stop(&ctx.data().data, &ctx.data().servitor, &name, ctx.author()).await;

	let embed = view_serv_act::stop_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command)]
pub async fn restart(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	let result =
		ctrl_serv_act::restart(&ctx.data().data, &ctx.data().servitor, &name, ctx.author()).await;

	let embed = view_serv_act::restart_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command)]
pub async fn reload(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	let result =
		ctrl_serv_act::reload(&ctx.data().data, &ctx.data().servitor, &name, ctx.author()).await;

	let embed = view_serv_act::reload_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command)]
pub async fn status(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	let result =
		ctrl_serv_act::status(&ctx.data().data, &ctx.data().servitor, &name, ctx.author()).await;

	let embed = view_serv_act::status_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}
