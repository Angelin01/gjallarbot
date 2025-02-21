use crate::controllers::wake_on_lan::wake::WakeError;
use crate::embeds;
use serenity::builder::CreateEmbed;

pub fn wake_embed(result: Result<(), WakeError>, machine_name: &str) -> CreateEmbed {
	match result {
		Ok(_) => embeds::success("Machine woken", format!("Machine {machine_name} woken")),
		Err(e) => match e {
			WakeError::Machine(_) => embeds::invalid_machine(machine_name),
			WakeError::Io { .. } => embeds::internal_error("Internal Error", format!("An unexpected error occurred while waking machine {machine_name}, please contact the bot's owner.")),
			WakeError::Unauthorized { .. } => embeds::error("Unauthorized", format!("You are not authorized to wake machine {machine_name}")),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::controllers::wake_on_lan::MachineError;
	use serenity::all::{Colour, UserId};

	#[test]
	fn given_wake_with_non_existing_machine_then_reply_with_invalid_machine() {
		let result = Err(WakeError::Machine(MachineError::DoesNotExist {
			machine_name: "NonExistentMachine".to_string(),
		}));

		let embed = wake_embed(result, "NonExistentMachine");

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonExistentMachine exists");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_wake_with_unauthorized_then_reply_with_unauthorized() {
		let result = Err(WakeError::Unauthorized {
			user: UserId::new(12345678901234567),
			machine_name: "SomeMachine".to_string(),
		});

		let embed = wake_embed(result, "SomeMachine");

		let expected_embed = CreateEmbed::default()
			.title(":x: Unauthorized")
			.colour(Colour(0xdd2e44))
			.description("You are not authorized to wake machine SomeMachine");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_wake_with_io_error_then_should_reply_with_unexpected_error() {
		let result = Err(WakeError::Io {
			kind: std::io::ErrorKind::PermissionDenied,
		});

		let embed = wake_embed(result, "SomeMachine");

		let expected_embed = CreateEmbed::default()
			.title(":tools: Internal Error")
			.colour(Colour(0xF4900C))
			.description("An unexpected error occurred while waking machine SomeMachine, please contact the bot's owner.");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_wake_success_then_should_reply_with_success_info() {
		let embed = wake_embed(Ok(()), "SomeMachine");

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Machine woken")
			.colour(Colour(0x77b255))
			.description("Machine SomeMachine woken");

		assert_eq!(embed, expected_embed);
	}
}
