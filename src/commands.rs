pub mod wake_on_lan;
#[cfg(debug_assertions)]
pub mod register;

use poise::Command;

use crate::data::{BotData, BotError};

pub fn commands() -> Vec<Command<BotData, BotError>> {
	let mut commands = vec![
		wake_on_lan::wake_on_lan(),
	];

	if cfg!(debug_assertions) {
		commands.push(register::register());
	}

	commands
}
