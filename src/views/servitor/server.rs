use crate::controllers::servitor::server::{
	AddServerError, DescribeServerError, ListServersError, RemoveServerError, ServerDescription,
};
use crate::embeds;
use crate::models::servitor::ServitorServer;
use crate::views::format_list;
use serenity::builder::CreateEmbed;
use std::collections::BTreeSet;

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
			AddServerError::Server(_) => embeds::error("Duplicate name", format!("A servitor server with name {server_name} already exists, try a different name")),
			AddServerError::Unexpected(_) => embeds::error("Error", "An unexpected error occurred while adding the server"),
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
		Err(e) => match e {
			RemoveServerError::Server(_) => embeds::invalid_servitor_server(server_name),
			RemoveServerError::Unexpected(_) => embeds::error("Error", "An unexpected error occurred while removing the server"),
		},
	}
}

pub fn list_servers_embed(result: Result<Vec<ServitorServer>, ListServersError>) -> CreateEmbed {
	match result {
		Ok(servers) => {
			let description = if servers.is_empty() {
				"There are no servitor servers configured".to_string()
			} else {
				let server_list = servers
					.iter()
					.map(|s| format!("- {}: {} - `{}`", s.name, s.servitor, s.unit_name))
					.collect::<Vec<String>>()
					.join("\n");
				format!("Configured servers:\n{server_list}")
			};

			embeds::info("Servitor server list", description)
		}
		Err(ListServersError::Unexpected(_)) => embeds::error("Error", "An unexpected error occurred while listing servers"),
	}
}

pub fn describe_server_embed(
	result: Result<ServerDescription, DescribeServerError>,
	name: &str,
) -> CreateEmbed {
	match result {
		Ok(desc) => {
			let user_set: BTreeSet<_> = desc.authorized_users.into_iter().collect();
			let role_set: BTreeSet<_> = desc.authorized_roles.into_iter().collect();
			let users = format_list(&user_set, |id| format!("<@{id}>"));
			let roles = format_list(&role_set, |id| format!("<@&{id}>"));

			embeds::info(
				format!("Servitor server {name}"),
				format!(
					"- Servitor: {}\n\
- Unit Name: `{}`\n\
- Authorized Users: {users}\n\
- Authorized Roles: {roles}",
					desc.server.servitor, desc.server.unit_name
				),
			)
		}
		Err(DescribeServerError::Server(_)) => embeds::invalid_servitor_server(name),
		Err(DescribeServerError::Unexpected(_)) => embeds::error("Error", "An unexpected error occurred while describing the server"),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::controllers::servitor::ServerError;
	use serenity::all::{Colour, RoleId, UserId};

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
		let embed = list_servers_embed(Ok(vec![]));

		let expected_embed = CreateEmbed::default()
			.title(":information_source: Servitor server list")
			.colour(Colour(0x55acee))
			.description("There are no servitor servers configured");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_some_servers_then_list_servers_replies_with_formatted_list() {
		let now = chrono::Utc::now();
		let servers = vec![
			ServitorServer {
				id: 1,
				name: "ServerOne".to_string(),
				servitor: "ServitorOne".to_string(),
				unit_name: "unit_one.service".to_string(),
				created_at: now,
				updated_at: now,
			},
			ServitorServer {
				id: 2,
				name: "ServerThree".to_string(),
				servitor: "ServitorThree".to_string(),
				unit_name: "unit_three.service".to_string(),
				created_at: now,
				updated_at: now,
			},
			ServitorServer {
				id: 3,
				name: "ServerTwo".to_string(),
				servitor: "ServitorTwo".to_string(),
				unit_name: "unit_two.service".to_string(),
				created_at: now,
				updated_at: now,
			},
		];

		let embed = list_servers_embed(Ok(servers));

		let expected_embed = CreateEmbed::default()
			.title(":information_source: Servitor server list")
			.colour(Colour(0x55acee))
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
		let result = Err(DescribeServerError::Server(ServerError::DoesNotExist {
			server_name: "NonExistingServer".to_string(),
		}));

		let embed = describe_server_embed(result, "NonExistingServer");

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Servitor server")
			.colour(Colour(0xdd2e44))
			.description("No Servitor server with name NonExistingServer exists");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_describe_server_with_no_users_or_roles_then_reply_with_server_info() {
		let now = chrono::Utc::now();
		let desc = ServerDescription {
			server: ServitorServer {
				id: 1,
				name: "SomeServer".to_string(),
				servitor: "foo".to_string(),
				unit_name: "bar".to_string(),
				created_at: now,
				updated_at: now,
			},
			authorized_users: vec![],
			authorized_roles: vec![],
		};

		let embed = describe_server_embed(Ok(desc), "SomeServer");

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
		let now = chrono::Utc::now();
		let desc = ServerDescription {
			server: ServitorServer {
				id: 1,
				name: "SomeServer".to_string(),
				servitor: "foo".to_string(),
				unit_name: "bar".to_string(),
				created_at: now,
				updated_at: now,
			},
			authorized_users: vec![
				UserId::new(12345678901234567),
				UserId::new(12345678901234568),
			],
			authorized_roles: vec![
				RoleId::new(98765432109876543),
				RoleId::new(98765432109876544),
			],
		};

		let embed = describe_server_embed(Ok(desc), "SomeServer");

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
