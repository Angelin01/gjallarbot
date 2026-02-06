use super::super::reply_no_mentions;
use super::{autocomplete_server_name, autocomplete_servitor_name};
use crate::bot::{BotError, Context};
use crate::controllers::servitor::server as ctrl_serv_srv;
use crate::db::DbConnection;
use crate::views::servitor::server as view_serv_srv;

#[poise::command(slash_command, owners_only, rename = "add-server")]
pub async fn add_server<D: DbConnection>(
	ctx: Context<'_, D>,
	#[description = "Server name"] name: String,
	#[description = "Servitor instance"]
	#[autocomplete = "autocomplete_servitor_name"]
	servitor: String,
	#[description = "Unit name"] unit_name: String,
) -> Result<(), BotError> {
	let mut conn = ctx.data().conn.lock().await;
	let servitor_names: Vec<String> = ctx.data().servitor.keys().cloned().collect();
	let result = ctrl_serv_srv::add_server(
		&mut *conn,
		&servitor_names,
		&name,
		&servitor,
		&unit_name,
	)
	.await;

	let embed = view_serv_srv::add_server_embed(result, &name, &servitor, &unit_name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command, owners_only, rename = "remove-server")]
pub async fn remove_server<D: DbConnection>(
	ctx: Context<'_, D>,
	#[description = "Server name"] name: String,
) -> Result<(), BotError> {
	let mut conn = ctx.data().conn.lock().await;
	let result = ctrl_serv_srv::remove_server(&mut *conn, &name).await;

	let embed = view_serv_srv::remove_server_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command, rename = "list-servers")]
pub async fn list_servers<D: DbConnection>(ctx: Context<'_, D>) -> Result<(), BotError> {
	let embed = ctrl_serv_srv::list_servers(&ctx.data().data, async |info| {
		view_serv_srv::list_servers_embed(info)
	})
	.await;

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command, rename = "describe-server")]
pub async fn describe_server<D: DbConnection>(
	ctx: Context<'_, D>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	name: String,
) -> Result<(), BotError> {
	let embed = ctrl_serv_srv::describe_server(&ctx.data().data, &name, async |info, name| {
		view_serv_srv::describe_server_embed(info, name)
	})
	.await;

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}
