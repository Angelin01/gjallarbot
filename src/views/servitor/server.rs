use crate::controllers::servitor::server::{AddServerError, RemoveServerError};
use crate::controllers::servitor::ServerError;
use crate::data::servitor::{ServerInfo, ServitorData};
use crate::embeds;
use crate::views::format_list;
use serenity::builder::CreateEmbed;

pub fn add_server_embed(
	result: Result<(), AddServerError>,
	server_name: &str,
	servitor: &str,
	unit_name: &str,
) -> CreateEmbed {
	match result {
		Ok(_) => embeds::success("Success", "Successfully added new server")
			.field("Name", server_name, true)
			.field("Servitor", servitor, true)
			.field("Unit Name", unit_name, true),
		Err(e) => match e {
			AddServerError::InvalidServitor { .. } => embeds::error("Invalid Servitor", format!("There is no such servitor instance with name {servitor}")),
			AddServerError::Server(_) => embeds::error("Duplicate name", format!("A servitor server with name {server_name} already exists, try a different name"))
		}
	}
}

pub fn remove_server_embed(
	result: Result<(), RemoveServerError>,
	server_name: &str,
) -> CreateEmbed {
	match result {
		Ok(_) => embeds::success("Success", "Successfully removed server").field(
			"Name",
			server_name,
			true,
		),
		Err(_) => embeds::invalid_servitor_server(server_name),
	}
}

pub fn list_servers_embed(servitor_data: &ServitorData) -> CreateEmbed {
	let description = if servitor_data.is_empty() {
		"There are no servitor servers configured".to_string()
	} else {
		let server_list = servitor_data
			.iter()
			.map(|(name, info)| format!("- {}: {} - `{}`", name, info.servitor, info.unit_name))
			.collect::<Vec<String>>()
			.join("\n");
		format!("Configured servers:\n{server_list}")
	};

	embeds::info("Servitor server list", description)
}

pub fn describe_server_embed(result: Result<&ServerInfo, ServerError>, name: &str) -> CreateEmbed {
	match result {
		Ok(server_info) => {
			let users = format_list(&server_info.authorized_users, |id| format!("<@{id}>"));
			let roles = format_list(&server_info.authorized_roles, |id| format!("<@&{id}>"));

			embeds::info(
				format!("Servitor server {name}"),
				format!(
					"- Servitor: {}\n\
- Unit Name: `{}`\n\
- Authorized Users: {users}\n\
- Authorized Roles: {roles}",
					server_info.servitor, server_info.unit_name
				),
			)
		}
		Err(_) => embeds::invalid_servitor_server(name),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::controllers::servitor::ServerError;
	use crate::data::servitor::ServerInfo;
	use serenity::all::{Colour, RoleId, UserId};
	use std::collections::BTreeSet;

	#[test]
	fn given_add_server_error_with_invalid_servitor_then_reply_with_invalid_servitor() {
		let result = Err(AddServerError::InvalidServitor {
			name: "foo".to_string(),
		});

		let embed = add_server_embed(result, "SomeServer", "foo", "bar");

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Servitor")
			.colour(Colour(0xdd2e44))
			.description("There is no such servitor instance with name foo");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_add_server_error_with_existing_server_then_reply_with_error_duplicate_name() {
		let result = Err(AddServerError::Server(ServerError::AlreadyExists {
			server_name: "SomeServer".to_string(),
		}));

		let embed = add_server_embed(result, "SomeServer", "foo", "bar");

		let expected_embed = CreateEmbed::default()
			.title(":x: Duplicate name")
			.colour(Colour(0xdd2e44))
			.description(
				"A servitor server with name SomeServer already exists, try a different name",
			);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_add_server_then_reply_with_success_info() {
		let embed = add_server_embed(Ok(()), "SomeServer", "foo", "bar");

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Success")
			.colour(Colour(0x77b255))
			.description("Successfully added new server")
			.field("Name", "SomeServer", true)
			.field("Servitor", "foo", true)
			.field("Unit Name", "bar", true);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_remove_server_with_does_not_exist_error_then_reply_with_non_existing_server() {
		let result = Err(RemoveServerError::Server(ServerError::DoesNotExist {
			server_name: "NonExistingServer".to_string(),
		}));

		let embed = remove_server_embed(result, "NonExistingServer");

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Servitor server")
			.colour(Colour(0xdd2e44))
			.description("No Servitor server with name NonExistingServer exists");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_remove_server_then_reply_with_success_info() {
		let embed = remove_server_embed(Ok(()), "SomeServer");

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Success")
			.colour(Colour(0x77b255))
			.description("Successfully removed server")
			.field("Name", "SomeServer", true);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_no_servers_then_list_servers_replies_with_empty_response() {
		let embed = list_servers_embed(&ServitorData::new());

		let expected_embed = CreateEmbed::default()
			.title(":information_source: Servitor server list")
			.colour(Colour(0x55acee))
			.description("There are no servitor servers configured");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_some_servers_then_list_servers_replies_with_formatted_list() {
		let data = ServitorData::from([
			(
				"ServerOne".to_string(),
				ServerInfo {
					servitor: "ServitorOne".to_string(),
					unit_name: "unit_one.service".to_string(),
					authorized_users: Default::default(),
					authorized_roles: Default::default(),
				},
			),
			(
				"ServerTwo".to_string(),
				ServerInfo {
					servitor: "ServitorTwo".to_string(),
					unit_name: "unit_two.service".to_string(),
					authorized_users: Default::default(),
					authorized_roles: Default::default(),
				},
			),
			(
				"ServerThree".to_string(),
				ServerInfo {
					servitor: "ServitorThree".to_string(),
					unit_name: "unit_three.service".to_string(),
					authorized_users: Default::default(),
					authorized_roles: Default::default(),
				},
			),
		]);

		let embed = list_servers_embed(&data);

		let expected_embed = CreateEmbed::default()
			.title(":information_source: Servitor server list")
			.colour(Colour(0x55acee))
			// Order is not important here, so we adjust the test to match output order based on data
			.description(
				"Configured servers:\n\
- ServerOne: ServitorOne - `unit_one.service`\n\
- ServerThree: ServitorThree - `unit_three.service`\n\
- ServerTwo: ServitorTwo - `unit_two.service`",
			);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_describe_server_with_non_exiting_server_then_reply_with_non_exiting_server() {
		let result = Err(ServerError::DoesNotExist {
			server_name: "NonExistingServer".to_string(),
		});

		let embed = describe_server_embed(result, "NonExistingServer");

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Servitor server")
			.colour(Colour(0xdd2e44))
			.description("No Servitor server with name NonExistingServer exists");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_describe_server_with_no_users_or_roles_then_reply_with_server_info() {
		let server_info = ServerInfo {
			servitor: "foo".to_string(),
			unit_name: "bar".to_string(),
			authorized_users: Default::default(),
			authorized_roles: Default::default(),
		};

		let embed = describe_server_embed(Ok(&server_info), "SomeServer");

		let expected_embed = CreateEmbed::default()
			.title(":information_source: Servitor server SomeServer")
			.colour(Colour(0x55acee))
			.description(
				"\
- Servitor: foo\n\
- Unit Name: `bar`\n\
- Authorized Users: None\n\
- Authorized Roles: None",
			);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_describe_server_with_users_and_roles_then_reply_with_server_info() {
		let server_info = ServerInfo {
			servitor: "foo".to_string(),
			unit_name: "bar".to_string(),
			authorized_users: BTreeSet::from([
				UserId::new(12345678901234567),
				UserId::new(12345678901234568),
			]),
			authorized_roles: BTreeSet::from([
				RoleId::new(98765432109876543),
				RoleId::new(98765432109876544),
			]),
		};

		let embed = describe_server_embed(Ok(&server_info), "SomeServer");

		let expected_embed = CreateEmbed::default()
			.title(":information_source: Servitor server SomeServer")
			.colour(Colour(0x55acee))
			.description(
				"\
- Servitor: foo\n\
- Unit Name: `bar`\n\
- Authorized Users: <@12345678901234567>, <@12345678901234568>\n\
- Authorized Roles: <@&98765432109876543>, <@&98765432109876544>",
			);

		assert_eq!(embed, expected_embed);
	}
}
