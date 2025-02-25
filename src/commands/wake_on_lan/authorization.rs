use super::super::reply_no_mentions;
use super::autocomplete_machine_name;
use crate::data::{BotError, Context};
use crate::{controllers, views};
use controllers::wake_on_lan::authorization as ctrl_wol_auth;
use poise::serenity_prelude::{Role, User};
use views::wake_on_lan::authorization as view_wol_auth;

#[poise::command(slash_command, owners_only, rename = "add-user")]
pub async fn add_user(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	machine_name: String,
	#[description = "User that be allowed wake this machine"] user: User,
) -> Result<(), BotError> {
	let result = ctrl_wol_auth::permit_user(&ctx.data().data, &machine_name, user.id).await;
	let embed = view_wol_auth::permit_user_embed(result, &machine_name, user.id);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command, owners_only, rename = "remove-user")]
pub async fn remove_user(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	machine_name: String,
	#[description = "User that will no longer be allowed wake this machine"] user: User,
) -> Result<(), BotError> {
	let result = ctrl_wol_auth::revoke_user(&ctx.data().data, &machine_name, user.id).await;
	let embed = view_wol_auth::revoke_user_embed(result, &machine_name, user.id);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command, owners_only, rename = "add-role")]
pub async fn add_role(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	machine_name: String,
	#[description = "Role that be allowed wake this machine"] role: Role,
) -> Result<(), BotError> {
	let result = ctrl_wol_auth::permit_role(&ctx.data().data, &machine_name, role.id).await;
	let embed = view_wol_auth::permit_role_embed(result, &machine_name, role.id);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}

#[poise::command(slash_command, owners_only, rename = "remove-role")]
pub async fn remove_role(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	machine_name: String,
	#[description = "Role that will no longer be allowed wake this machine"] role: Role,
) -> Result<(), BotError> {
	let result = ctrl_wol_auth::revoke_role(&ctx.data().data, &machine_name, role.id).await;
	let embed = view_wol_auth::revoke_role_embed(result, &machine_name, role.id);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}
