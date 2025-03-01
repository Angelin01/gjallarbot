use crate::controllers::servitor::server::{AddServerError, RemoveServerError};
use crate::data::servitor::ServitorData;
use crate::embeds;
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
		Err(_) => embeds::error(
			"Invalid Server",
			format!("No servitor server with name {server_name} exists"),
		),
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::controllers::servitor::ServerError;
	use crate::data::servitor::ServerInfo;
	use serenity::all::Colour;

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
			.title(":x: Invalid Server")
			.colour(Colour(0xdd2e44))
			.description("No servitor server with name NonExistingServer exists");

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
}
