use super::super::reply_no_mentions;
use super::autocomplete_server_name;
use crate::bot::{BotError, Context};

use crate::controllers::servitor::action as ctrl_serv_act;
use crate::db::DbConnection;
use crate::views::servitor::action as view_serv_act;

#[poise::command(slash_command)]
pub async fn start<D: DbConnection>(
	ctx: Context<'_, D>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	let result = {
		let mut conn = ctx.data().conn.lock().await;
		ctrl_serv_act::start(&mut *conn, &ctx.data().servitor, &name, ctx.author(), ctx.author_member().await.as_deref()).await
	};

	let embed = view_serv_act::start_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command)]
pub async fn stop<D: DbConnection>(
	ctx: Context<'_, D>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	let result = {
		let mut conn = ctx.data().conn.lock().await;
		ctrl_serv_act::stop(&mut *conn, &ctx.data().servitor, &name, ctx.author(), ctx.author_member().await.as_deref()).await
	};

	let embed = view_serv_act::stop_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command)]
pub async fn restart<D: DbConnection>(
	ctx: Context<'_, D>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	let result = {
		let mut conn = ctx.data().conn.lock().await;
		ctrl_serv_act::restart(&mut *conn, &ctx.data().servitor, &name, ctx.author(), ctx.author_member().await.as_deref()).await
	};

	let embed = view_serv_act::restart_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command)]
pub async fn reload<D: DbConnection>(
	ctx: Context<'_, D>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	let result = {
		let mut conn = ctx.data().conn.lock().await;
		ctrl_serv_act::reload(&mut *conn, &ctx.data().servitor, &name, ctx.author(), ctx.author_member().await.as_deref()).await
	};

	let embed = view_serv_act::reload_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command)]
pub async fn status<D: DbConnection>(
	ctx: Context<'_, D>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	let result = {
		let mut conn = ctx.data().conn.lock().await;
		ctrl_serv_act::status(&mut *conn, &ctx.data().servitor, &name, ctx.author(), ctx.author_member().await.as_deref()).await
	};

	let embed = view_serv_act::status_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}
