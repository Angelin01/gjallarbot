use super::super::format_list;
use crate::controllers::wake_on_lan::machine::{AddMachineError, RemoveMachineError};
use crate::controllers::wake_on_lan::MachineError;
use crate::data::wake_on_lan::{WakeOnLanData, WakeOnLanMachineInfo};
use crate::embeds;
use serenity::builder::CreateEmbed;

pub fn add_machine_embed(
	result: Result<(), AddMachineError>,
	machine_name: &str,
	mac_address: &str,
) -> CreateEmbed {
	match result {
		Ok(_) => embeds::success("Success", "Successfully added new machine!")
			.field("Name", machine_name, true)
			.field("MAC Address", mac_address, true),
		Err(e) => match e {
			AddMachineError::Machine(_) => embeds::error(
				"Duplicate name",
				format!("A machine with name {machine_name} already exists, try a different name"),
			),
			AddMachineError::InvalidMac(m) => embeds::error(
				"Invalid MAC Address",
				format!("Mac address {mac_address} is invalid: {m}"),
			),
		},
	}
}

pub fn remove_machine_embed(
	result: Result<(), RemoveMachineError>,
	machine_name: &str,
) -> CreateEmbed {
	match result {
		Ok(_) => embeds::success("Success", "Successfully removed machine!").field(
			"Name",
			machine_name,
			true,
		),
		Err(_) => embeds::invalid_machine(machine_name),
	}
}

pub fn list_machines_embed(wake_on_lan_data: &WakeOnLanData) -> CreateEmbed {
	let description = if wake_on_lan_data.is_empty() {
		"There are no machines configured".to_string()
	} else {
		let machine_list = wake_on_lan_data
			.iter()
			.map(|m| format!("- {}: `{}`", m.0, m.1.mac))
			.collect::<Vec<String>>()
			.join("\n");
		format!("Configured machines:\n{machine_list}")
	};

	embeds::info("Machine list", description)
}

pub fn describe_machine_embed(
	result: Result<&WakeOnLanMachineInfo, MachineError>,
	machine_name: &str,
) -> CreateEmbed {
	match result {
		Ok(machine_info) => {
			let users = format_list(&machine_info.authorized_users, |id| format!("<@{id}>"));
			let roles = format_list(&machine_info.authorized_roles, |id| format!("<@&{id}>"));

			embeds::info(
				format!("Machine {machine_name}"),
				format!(
					"- MAC Address: `{}`\n\
                     - Authorized Users: {}\n\
                     - Authorized Roles: {}",
					machine_info.mac, users, roles
				),
			)
		},
		Err(_) => embeds::invalid_machine(machine_name),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::controllers::wake_on_lan::MachineError;
	use crate::data::wake_on_lan::WakeOnLanMachineInfo;
	use crate::errors::InvalidMacError;
	use crate::services::wake_on_lan::MacAddress;
	use serenity::all::{Colour, RoleId, UserId};
	use std::collections::BTreeSet;

	#[test]
	fn given_add_machine_error_with_existing_machine_then_reply_with_error_existing_machine() {
		let result = Err(AddMachineError::Machine(MachineError::AlreadyExists {
			machine_name: "SomeMachine".into(),
		}));

		let embed = add_machine_embed(result, "SomeMachine", "01:02:03:04:05:06");

		let expected_embed = CreateEmbed::default()
			.title(":x: Duplicate name")
			.colour(Colour(0xdd2e44))
			.description("A machine with name SomeMachine already exists, try a different name");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_add_machine_error_with_invalid_mac_parts_then_reply_with_error_invalid_parts() {
		let result = Err(AddMachineError::InvalidMac(
			InvalidMacError::WrongPartCount {
				expected: 6,
				actual: 1,
			},
		));

		let embed = add_machine_embed(result, "SomeMachine", "invalid_mac");

		let expected_embed = CreateEmbed::default()
            .title(":x: Invalid MAC Address")
            .colour(Colour(0xdd2e44))
            .description("Mac address invalid_mac is invalid: Expected 6 parts in MAC address separated by `:`, but got 1");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_add_machine_error_with_invalid_mac_hex_then_reply_with_error_invalid_hex() {
		let result = Err(AddMachineError::InvalidMac(
			InvalidMacError::InvalidHexString("PP".into()),
		));

		let embed = add_machine_embed(result, "SomeMachine", "01:02:03:04:05:PP");

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid MAC Address")
			.colour(Colour(0xdd2e44))
			.description("Mac address 01:02:03:04:05:PP is invalid: Invalid hexadecimal value PP");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_add_machine_then_reply_with_success_info() {
		let embed = add_machine_embed(Ok(()), "SomeMachine", "01:02:03:04:05:06");

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Success")
			.colour(Colour(0x77b255))
			.description("Successfully added new machine!")
			.field("Name", "SomeMachine", true)
			.field("MAC Address", "01:02:03:04:05:06", true);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_remove_machine_error_with_non_existing_machine_then_reply_with_non_existing_machine() {
		let result = Err(RemoveMachineError::Machine(MachineError::DoesNotExist {
			machine_name: "NonExistentMachine".to_string(),
		}));

		let embed = remove_machine_embed(result, "NonExistentMachine");

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonExistentMachine exists");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_remove_machine_then_reply_with_success_info() {
		let embed = remove_machine_embed(Ok(()), "SomeMachine");

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Success")
			.colour(Colour(0x77b255))
			.description("Successfully removed machine!")
			.field("Name", "SomeMachine", true);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_no_added_machines_then_list_machines_replies_with_empty_response() {
		let embed = list_machines_embed(&WakeOnLanData::new());

		let expected_embed = CreateEmbed::default()
			.title(":information_source: Machine list")
			.colour(Colour(0x55acee))
			.description("There are no machines configured");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_some_machines_then_list_machines_replies_formatted_list() {
		let data = WakeOnLanData::from([
			(
				"MachineOne".to_string(),
				WakeOnLanMachineInfo {
					mac: MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
					authorized_users: Default::default(),
					authorized_roles: Default::default(),
				},
			),
			(
				"MachineTwo".to_string(),
				WakeOnLanMachineInfo {
					mac: MacAddress([0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C]),
					authorized_users: Default::default(),
					authorized_roles: Default::default(),
				},
			),
			(
				"MachineThree".to_string(),
				WakeOnLanMachineInfo {
					mac: MacAddress([0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12]),
					authorized_users: Default::default(),
					authorized_roles: Default::default(),
				},
			),
		]);

		let embed = list_machines_embed(&data);

		let expected_embed = CreateEmbed::default()
			.title(":information_source: Machine list")
			.colour(Colour(0x55acee))
			.description(
				"Configured machines:\n\
- MachineOne: `01:02:03:04:05:06`\n\
- MachineThree: `0D:0E:0F:10:11:12`\n\
- MachineTwo: `07:08:09:0A:0B:0C`",
			);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_describe_machine_with_non_existing_machine_then_reply_with_non_existing_machine() {
		let result = Err(MachineError::DoesNotExist {
			machine_name: "NonExistentMachine".to_string(),
		});

		let embed = describe_machine_embed(result, "NonExistentMachine");

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonExistentMachine exists");

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_describe_machine_with_no_users_or_roles_then_reply_with_machine_info() {
		let machine_info = WakeOnLanMachineInfo {
			mac: MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
			authorized_users: Default::default(),
			authorized_roles: Default::default(),
		};

		let embed = describe_machine_embed(Ok(&machine_info), "SomeMachine");

		let expected_embed = embeds::info(
			"Machine SomeMachine",
			"- MAC Address: `01:02:03:04:05:06`\n\
         - Authorized Users: None\n\
         - Authorized Roles: None",
		);

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_successful_describe_machine_with_users_and_roles_then_reply_with_machine_info() {
		let machine_info = WakeOnLanMachineInfo {
			mac: MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
			authorized_users: BTreeSet::from([
				UserId::new(12345678901234567),
				UserId::new(12345678901234568),
			]),
			authorized_roles: BTreeSet::from([
				RoleId::new(98765432109876543),
				RoleId::new(98765432109876544),
			]),
		};

		let embed = describe_machine_embed(Ok(&machine_info), "SomeMachine");

		let expected_embed = embeds::info(
			"Machine SomeMachine",
			"- MAC Address: `01:02:03:04:05:06`\n\
         - Authorized Users: <@12345678901234567>, <@12345678901234568>\n\
         - Authorized Roles: <@&98765432109876543>, <@&98765432109876544>",
		);

		assert_eq!(embed, expected_embed);
	}
}
