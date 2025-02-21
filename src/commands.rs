pub mod wake_on_lan;
#[cfg(debug_assertions)]
pub mod register;

use poise::{Command, CreateReply, ReplyHandle};
use serenity::all::{CreateAllowedMentions, CreateEmbed};
use crate::data::{BotData, BotError, Context};

pub fn commands() -> Vec<Command<BotData, BotError>> {
	let commands = vec![
		wake_on_lan::wake_on_lan(),
		#[cfg(debug_assertions)] register::register(),
	];

	commands
}

async fn reply_no_mentions<'a>(ctx: Context<'a>, embed: CreateEmbed) -> Result<ReplyHandle<'a>, BotError> {
	Ok(ctx.send(
		CreateReply::default()
			.embed(embed)
			.allowed_mentions(CreateAllowedMentions::default().empty_users().empty_roles())
	).await?)
}
