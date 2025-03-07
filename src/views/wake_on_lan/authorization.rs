use crate::controllers::wake_on_lan::authorization::{AddPermissionError, RemovePermissionError};
use crate::embeds;
use serenity::all::{CreateEmbed, RoleId, UserId};

pub fn permit_user_embed(
	result: Result<(), AddPermissionError>,
	machine_name: &str,
	user_id: UserId,
) -> CreateEmbed {
	match result {
		Ok(_) => embeds::success(
			"User permitted",
			"Successfully permitted user to wake the machine!",
		)
		.field("Machine", machine_name, true)
		.field("User", format!("<@{user_id}>"), true),
		Err(e) => match e {
			AddPermissionError::Machine(_) => embeds::invalid_machine(machine_name),
			AddPermissionError::AlreadyAuthorized { .. } => embeds::error(
				"User already permitted",
				format!("User <@{user_id}> is already permitted to wake machine {machine_name}"),
			),
		},
	}
}

pub fn revoke_user_embed(
	result: Result<(), RemovePermissionError>,
	machine_name: &str,
	user_id: UserId,
) -> CreateEmbed {
	match result {
		Ok(_) => embeds::success(
			"User permission revoked",
			"Successfully revoked user's permission to wake the machine!",
		)
		.field("Machine", machine_name, true)
		.field("User", format!("<@{user_id}>"), true),
		Err(e) => match e {
			RemovePermissionError::Machine(_) => embeds::invalid_machine(machine_name),
			RemovePermissionError::AlreadyNotAuthorized { .. } => embeds::error(
				"User not permitted",
				format!(
					"User <@{user_id}> is already not permitted to wake machine {machine_name}"
				),
			),
		},
	}
}

pub fn permit_role_embed(
	result: Result<(), AddPermissionError>,
	machine_name: &str,
	role_id: RoleId,
) -> CreateEmbed {
	match result {
		Ok(_) => embeds::success(
			"Role permitted",
			"Successfully permitted role to wake the machine!",
		)
		.field("Machine", machine_name, true)
		.field("Role", format!("<@{role_id}>"), true),
		Err(e) => match e {
			AddPermissionError::Machine(_) => embeds::invalid_machine(machine_name),
			AddPermissionError::AlreadyAuthorized { .. } => embeds::error(
				"Role already permitted",
				format!("Role <@{role_id}> is already permitted to wake machine {machine_name}"),
			),
		},
	}
}

pub fn revoke_role_embed(
	result: Result<(), RemovePermissionError>,
	machine_name: &str,
	role_id: RoleId,
) -> CreateEmbed {
	match result {
		Ok(_) => embeds::success(
			"Role permission revoked",
			"Successfully revoked role's permission to wake the machine!",
		)
		.field("Machine", machine_name, true)
		.field("Role", format!("<@{role_id}>"), true),
		Err(e) => match e {
			RemovePermissionError::Machine(_) => embeds::invalid_machine(machine_name),
			RemovePermissionError::AlreadyNotAuthorized { .. } => embeds::error(
				"Role not permitted",
				format!(
					"Role <@{role_id}> is already not permitted to wake machine {machine_name}"
				),
			),
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::controllers::wake_on_lan::MachineError;
	use serenity::all::Colour;
	use crate::controllers::DiscordEntity;

	#[test]
	fn given_permit_user_error_with_nonexistent_machine_then_reply_with_error_no_machine() {
		let result = Err(AddPermissionError::Machine(MachineError::DoesNotExist {
			machine_name: "NonExistentMachine".to_string(),
		}));
		let embed = permit_user_embed(result, "NonExistentMachine", UserId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonExistentMachine exists");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_permit_user_error_with_already_authorized_then_reply_with_error_already_authorized() {
		let result = Err(AddPermissionError::AlreadyAuthorized {
			machine_name: "SomeMachine".to_string(),
			entity: DiscordEntity::User(UserId::new(12345678901234567)),
		});
		let embed = permit_user_embed(result, "SomeMachine", UserId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":x: User already permitted")
			.colour(Colour(0xdd2e44))
			.description(
				"User <@12345678901234567> is already permitted to wake machine SomeMachine",
			);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_permit_user_then_should_reply_with_success_info() {
		let embed = permit_user_embed(Ok(()), "SomeMachine", UserId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: User permitted")
			.colour(Colour(0x77b255))
			.description("Successfully permitted user to wake the machine!")
			.field("Machine", "SomeMachine", true)
			.field("User", "<@12345678901234567>", true);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_revoke_user_error_with_nonexistent_machine_then_reply_with_error_no_machine() {
		let result = Err(RemovePermissionError::Machine(MachineError::DoesNotExist {
			machine_name: "NonExistentMachine".to_string(),
		}));
		let embed = revoke_user_embed(result, "NonExistentMachine", UserId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonExistentMachine exists");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_revoke_user_error_with_already_unauthorized_then_reply_with_already_unauthorized() {
		let result = Err(RemovePermissionError::AlreadyNotAuthorized {
			machine_name: "SomeMachine".to_string(),
			entity: DiscordEntity::User(UserId::new(76543210987654321)),
		});
		let embed = revoke_user_embed(result, "SomeMachine", UserId::new(76543210987654321));

		let expected_embed = CreateEmbed::default()
			.title(":x: User not permitted")
			.colour(Colour(0xdd2e44))
			.description(
				"User <@76543210987654321> is already not permitted to wake machine SomeMachine",
			);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_revoke_user_then_should_reply_with_success_info() {
		let embed = revoke_user_embed(Ok(()), "SomeMachine", UserId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: User permission revoked")
			.colour(Colour(0x77b255))
			.description("Successfully revoked user's permission to wake the machine!")
			.field("Machine", "SomeMachine", true)
			.field("User", "<@12345678901234567>", true);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_permit_role_error_with_nonexistent_machine_then_reply_with_error_no_machine() {
		let result = Err(AddPermissionError::Machine(MachineError::DoesNotExist {
			machine_name: "NonExistentMachine".to_string(),
		}));
		let embed = permit_role_embed(result, "NonExistentMachine", RoleId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonExistentMachine exists");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_permit_role_error_with_already_authorized_then_reply_with_error_already_authorized() {
		let result = Err(AddPermissionError::AlreadyAuthorized {
			machine_name: "SomeMachine".to_string(),
			entity: DiscordEntity::Role(RoleId::new(12345678901234567)),
		});
		let embed = permit_role_embed(result, "SomeMachine", RoleId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":x: Role already permitted")
			.colour(Colour(0xdd2e44))
			.description(
				"Role <@12345678901234567> is already permitted to wake machine SomeMachine",
			);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_permit_role_then_should_reply_with_success_info() {
		let embed = permit_role_embed(Ok(()), "SomeMachine", RoleId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Role permitted")
			.colour(Colour(0x77b255))
			.description("Successfully permitted role to wake the machine!")
			.field("Machine", "SomeMachine", true)
			.field("Role", "<@12345678901234567>", true);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_revoke_role_error_with_nonexistent_machine_then_reply_with_error_no_machine() {
		let result = Err(RemovePermissionError::Machine(MachineError::DoesNotExist {
			machine_name: "NonExistentMachine".to_string(),
		}));
		let embed = revoke_role_embed(result, "NonExistentMachine", RoleId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonExistentMachine exists");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_revoke_role_error_with_already_unauthorized_then_reply_with_already_unauthorized() {
		let result = Err(RemovePermissionError::AlreadyNotAuthorized {
			machine_name: "SomeMachine".to_string(),
			entity: DiscordEntity::Role(RoleId::new(76543210987654321)),
		});
		let embed = revoke_role_embed(result, "SomeMachine", RoleId::new(76543210987654321));

		let expected_embed = CreateEmbed::default()
			.title(":x: Role not permitted")
			.colour(Colour(0xdd2e44))
			.description(
				"Role <@76543210987654321> is already not permitted to wake machine SomeMachine",
			);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_revoke_role_then_should_reply_with_success_info() {
		let embed = revoke_role_embed(Ok(()), "SomeMachine", RoleId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Role permission revoked")
			.colour(Colour(0x77b255))
			.description("Successfully revoked role's permission to wake the machine!")
			.field("Machine", "SomeMachine", true)
			.field("Role", "<@12345678901234567>", true);

		assert_eq!(embed, expected_embed);
	}
}
