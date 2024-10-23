pub mod wake_on_lan;
#[cfg(debug_assertions)]
pub mod register;

use poise::Command;

use crate::data::{BotData, BotError};

pub fn commands() -> Vec<Command<BotData, BotError>> {
	let commands = vec![
		wake_on_lan::wake_on_lan(),
		#[cfg(debug_assertions)] register::register(),
	];

	commands
}
