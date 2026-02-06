use crate::bot::{BotError, Context};
use crate::commands::reply_no_mentions;
use crate::commands::wake_on_lan::autocomplete_machine_name;
use crate::services::wake_on_lan::UdpMagicPacketSender;
use crate::{controllers, views};
use crate::db::DbConnection;

#[poise::command(slash_command)]
pub async fn wake<D: DbConnection>(
	ctx: Context<'_, D>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	name: String,
) -> Result<(), BotError> {
	const SENDER: UdpMagicPacketSender = UdpMagicPacketSender {};

	let mut conn = ctx.data().conn.lock().await;

	let result = controllers::wake_on_lan::wake::wake(
		&mut *conn,
		ctx.author(),
		ctx.author_member().await.as_deref(),
		&name,
		&SENDER,
	)
	.await;

	let embed = views::wake_on_lan::wake::wake_embed(result, &name);

	reply_no_mentions(ctx, embed).await?;

	Ok(())
}
