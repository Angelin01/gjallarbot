use super::super::reply_no_mentions;
use super::autocomplete_server_name;
use crate::bot::{BotError, Context};
use crate::controllers::servitor::authorization as ctrl_serv_auth;
use crate::views::servitor::authorization as view_serv_auth;
use serenity::all::{Role, User};

#[poise::command(slash_command, owners_only, rename = "add-user")]
pub async fn add_user(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	server: String,
	#[description = "User that be allowed operate this server"] user: User,
) -> Result<(), BotError> {
	let result = ctrl_serv_auth::permit_user(&ctx.data().data, &server, user.id).await;
	let embed = view_serv_auth::permit_user_embed(result, &server, user.id);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command, owners_only, rename = "remove-user")]
pub async fn remove_user(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	server: String,
	#[description = "User that will no longer be allowed operate this server"] user: User,
) -> Result<(), BotError> {
	let result = ctrl_serv_auth::revoke_user(&ctx.data().data, &server, user.id).await;
	let embed = view_serv_auth::revoke_user_embed(result, &server, user.id);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command, owners_only, rename = "add-role")]
pub async fn add_role(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	server: String,
	#[description = "Role that be allowed operate this server"] role: Role,
) -> Result<(), BotError> {
	let result = ctrl_serv_auth::permit_role(&ctx.data().data, &server, role.id).await;
	let embed = view_serv_auth::permit_role_embed(result, &server, role.id);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command, owners_only, rename = "remove-role")]
pub async fn remove_role(
	ctx: Context<'_>,
	#[description = "Server name"]
	#[autocomplete = "autocomplete_server_name"]
	server: String,
	#[description = "Role that will no longer be allowed operate this server"] role: Role,
) -> Result<(), BotError> {
	let result = ctrl_serv_auth::revoke_role(&ctx.data().data, &server, role.id).await;
	let embed = view_serv_auth::revoke_role_embed(result, &server, role.id);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}
