use crate::controllers::servitor::authorization::{AddPermissionError, RemovePermissionError};
use crate::embeds;
use serenity::all::UserId;
use serenity::builder::CreateEmbed;

pub fn permit_user_embed(
	result: Result<(), AddPermissionError>,
	server_name: &str,
	user_id: UserId,
) -> CreateEmbed {
	match result {
		Ok(_) => embeds::success(
			"User permitted",
			"Successfully permitted user to operate the Servitor server!",
		)
		.field("Servitor server", server_name, true)
		.field("User", format!("<@{user_id}>"), true),
		Err(e) => match e {
			AddPermissionError::Server(_) => embeds::invalid_servitor_server(server_name),
			AddPermissionError::AlreadyAuthorized { .. } => embeds::error(
				"User already permitted",
				format!("User <@{user_id}> is already permitted to operate Servitor server {server_name}"),
			),
		},
	}
}

pub fn revoke_user_embed(
	result: Result<(), RemovePermissionError>,
	server_name: &str,
	user_id: UserId,
) -> CreateEmbed {
	match result {
		Ok(_) => embeds::success(
			"User permission revoked",
			"Successfully revoked user's permission to operate the Servitor server!",
		)
		.field("Servitor server", server_name, true)
		.field("User", format!("<@{user_id}>"), true),
		Err(e) => match e {
			RemovePermissionError::Server(_) => embeds::invalid_servitor_server(server_name),
			RemovePermissionError::AlreadyNotAuthorized { .. } => embeds::error(
				"User not permitted",
				format!(
					"User <@{user_id}> is already not permitted to operate Servitor server {server_name}"
				),
			),
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::controllers::servitor::ServerError;
	use crate::controllers::DiscordEntity;
	use serenity::all::Colour;

	#[test]
	fn given_permit_user_error_with_nonexistent_server_then_reply_with_error_no_server() {
		let result = Err(AddPermissionError::Server(ServerError::DoesNotExist {
			server_name: "NonExistingServer".to_string(),
		}));
		let embed = permit_user_embed(result, "NonExistingServer", UserId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Servitor server")
			.colour(Colour(0xdd2e44))
			.description("No Servitor server with name NonExistingServer exists");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_permit_user_error_with_already_authorized_then_reply_with_error_already_authorized() {
		let result = Err(AddPermissionError::AlreadyAuthorized {
			server_name: "SomeServer".to_string(),
			entity: DiscordEntity::User(UserId::new(12345678901234567)),
		});
		let embed = permit_user_embed(result, "SomeServer", UserId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":x: User already permitted")
			.colour(Colour(0xdd2e44))
			.description(
				"User <@12345678901234567> is already permitted to operate Servitor server SomeServer",
			);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_permit_user_then_should_reply_with_success_info() {
		let embed = permit_user_embed(Ok(()), "SomeServer", UserId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: User permitted")
			.colour(Colour(0x77b255))
			.description("Successfully permitted user to operate the Servitor server!")
			.field("Servitor server", "SomeServer", true)
			.field("User", "<@12345678901234567>", true);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_revoke_user_error_with_nonexistent_server_then_reply_with_error_no_server() {
		let result = Err(RemovePermissionError::Server(ServerError::DoesNotExist {
			server_name: "NonExistentServer".to_string(),
		}));
		let embed = revoke_user_embed(result, "NonExistentServer", UserId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Servitor server")
			.colour(Colour(0xdd2e44))
			.description("No Servitor server with name NonExistentServer exists");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_revoke_user_error_with_already_unauthorized_then_reply_with_already_unauthorized() {
		let result = Err(RemovePermissionError::AlreadyNotAuthorized {
			server_name: "SomeServer".to_string(),
			entity: DiscordEntity::User(UserId::new(76543210987654321)),
		});
		let embed = revoke_user_embed(result, "SomeServer", UserId::new(76543210987654321));

		let expected_embed = CreateEmbed::default()
			.title(":x: User not permitted")
			.colour(Colour(0xdd2e44))
			.description(
				"User <@76543210987654321> is already not permitted to operate Servitor server SomeServer",
			);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_revoke_user_then_should_reply_with_success_info() {
		let embed = revoke_user_embed(Ok(()), "SomeServer", UserId::new(12345678901234567));

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: User permission revoked")
			.colour(Colour(0x77b255))
			.description("Successfully revoked user's permission to operate the Servitor server!")
			.field("Servitor server", "SomeServer", true)
			.field("User", "<@12345678901234567>", true);

		assert_eq!(embed, expected_embed);
	}
}
