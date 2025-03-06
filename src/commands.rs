mod wake_on_lan;
#[cfg(debug_assertions)]
mod register;
mod servitor;

use poise::{Command, CreateReply, ReplyHandle};
use serenity::all::{CreateAllowedMentions, CreateEmbed};
use crate::bot::{BotError, BotState, Context};
pub fn commands() -> Vec<Command<BotState, BotError>> {
	let commands = vec![
		wake_on_lan::wake_on_lan(),
		servitor::servitor(),
		#[cfg(debug_assertions)] register::register(),
	];

	commands
}

const DISCORD_MAX_AUTOCOMPLETE_CHOICES: usize = 25;

async fn reply_no_mentions<'a>(ctx: Context<'a>, embed: CreateEmbed) -> Result<ReplyHandle<'a>, BotError> {
	Ok(ctx.send(
		CreateReply::default()
			.embed(embed)
			.allowed_mentions(CreateAllowedMentions::default().empty_users().empty_roles())
	).await?)
}
