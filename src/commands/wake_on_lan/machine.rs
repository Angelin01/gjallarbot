use poise::CreateReply;
use poise::serenity_prelude::{CreateAllowedMentions, CreateEmbed};
use super::autocomplete_machine_name;
use crate::data::{BotData, BotError, Context};
use crate::data::wake_on_lan::WakeOnLanMachineInfo;
use crate::embeds;
use crate::services::wake_on_lan::MacAddress;

// Adds a new machine that can be woken up
#[poise::command(slash_command, owners_only, rename = "add-machine")]
pub async fn add_machine(
	ctx: Context<'_>,
	#[description = "Machine name"] name: String,
	#[description = "Machine MAC Address as hex digits separated by :"] mac: String,
) -> Result<(), BotError> {
	let embed = process_add_machine(ctx.data(), name, mac).await?;

	ctx.send(CreateReply::default().embed(embed)).await?;

	Ok(())
}

async fn process_add_machine(data: &BotData, name: String, mac: String) -> Result<CreateEmbed, BotError> {
	{
		let read = data.read().await;
		if read.wake_on_lan.contains_key(&name) {
			return Ok(embeds::error(
				"Duplicate name",
				format!("A machine with name {name} already exists, try a different name"),
			));
		}
	}

	let mac_address: MacAddress = match mac.parse() {
		Ok(v) => v,
		Err(e) => {
			let err_msg = e.to_string();
			return Ok(embeds::error(
				"Invalid MAC Address",
				format!("Mac address {mac} is invalid: {err_msg}"),
			));
		}
	};

	let name_field = name.clone();
	let mac_field = mac.to_string();

	{
		let mut lock = data.write().await;
		let mut data_write = lock.write();
		data_write.wake_on_lan.insert(name, WakeOnLanMachineInfo {
			mac: mac_address,
			authorized_users: Default::default(),
			authorized_roles: Default::default(),
		});
	}

	let embed = embeds::success("Success", "Successfully added new machine!")
		.field("Name", name_field, true)
		.field("MAC Address", mac_field, true);

	Ok(embed)
}

// Removes a previously configured machine
#[poise::command(slash_command, owners_only, rename = "remove-machine")]
pub async fn remove_machine(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	name: String,
) -> Result<(), BotError> {
	let embed = process_remove_machine(ctx.data(), name).await?;

	ctx.send(CreateReply::default().embed(embed)).await?;

	Ok(())
}

async fn process_remove_machine(data: &BotData, name: String) -> Result<CreateEmbed, BotError> {
	{
		let read = data.read().await;
		if !read.wake_on_lan.contains_key(&name) {
			return Ok(embeds::invalid_machine(&name));
		}
	}

	let name_field = name.clone();

	{
		let mut lock = data.write().await;
		let mut data_write = lock.write();
		data_write.wake_on_lan.remove(&name);
	}

	let embed = embeds::success("Success", "Successfully removed machine!")
		.field("Name", name_field, true);

	Ok(embed)
}

/// Lists all configured machines and their MAC Address
#[poise::command(slash_command, rename = "list-machines")]
pub async fn list_machines(
	ctx: Context<'_>,
) -> Result<(), BotError> {
	let embed = process_list_machines(ctx.data()).await?;

	ctx.send(
		CreateReply::default()
			.embed(embed)
			.allowed_mentions(CreateAllowedMentions::default().empty_users().empty_roles())
	).await?;

	Ok(())
}

async fn process_list_machines(data: &BotData) -> Result<CreateEmbed, BotError> {
	let read = data.read().await;
	if read.wake_on_lan.is_empty() {
		return Ok(embeds::info(
			"Machine list",
			"There are no machines configured",
		));
	}

	let machine_list = read.wake_on_lan.iter()
		.map(|m| format!("- {}: `{}`", m.0, m.1.mac))
		.collect::<Vec<String>>()
		.join("\n");

	Ok(embeds::info(
		"Machine list",
		format!("Configured machines:\n{machine_list}"),
	))
}

/// Displays full information about a specific machine
#[poise::command(slash_command, rename = "describe-machine")]
pub async fn describe_machine(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	machine_name: String,
) -> Result<(), BotError> {
	let embed = process_describe_machine(ctx.data(), machine_name).await?;

	ctx.send(CreateReply::default().embed(embed)).await?;

	Ok(())
}

async fn process_describe_machine(data: &BotData, machine_name: String) -> Result<CreateEmbed, BotError> {
	let data_read = data.read().await;

	let machine_info = match data_read.wake_on_lan.get(&machine_name) {
		Some(info) => info,
		None => return Ok(embeds::invalid_machine(&machine_name)),
	};

	let users = if machine_info.authorized_users.is_empty() {
		"None".to_string()
	}
	else {
		machine_info.authorized_users
			.iter()
			.map(|user_id| format!("<@{user_id}>"))
			.collect::<Vec<String>>()
			.join(", ")
	};

	let roles = if machine_info.authorized_roles.is_empty() {
		"None".to_string()
	}
	else {
		machine_info.authorized_roles
			.iter()
			.map(|role_id| format!("<@&{role_id}>"))
			.collect::<Vec<String>>()
			.join(", ")
	};

	let mac = &machine_info.mac;

	let embed = embeds::info(
		&format!("Machine {machine_name}"),
		&format!(
			"- MAC Address: `{mac}`\n\
             - Authorized Users: {users}\n\
             - Authorized Roles: {roles}",
		),
	);

	Ok(embed)
}


#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::BTreeMap;
	use poise::serenity_prelude::Colour;
	use serde_json::json;
	use crate::data::tests::mock_data;

	#[tokio::test]
	async fn given_duplicate_name_then_returns_error_and_does_not_update_data() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"SomeMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		let result = process_add_machine(
			&data,
			"SomeMachine".into(),
			"00:00:00:00:00:01".into(),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Duplicate name")
			.colour(Colour(0xdd2e44))
			.description("A machine with name SomeMachine already exists, try a different name");

		let mut expected_data = BTreeMap::new();
		expected_data.insert("SomeMachine".to_string(), WakeOnLanMachineInfo {
			mac: MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
			authorized_users: Default::default(),
			authorized_roles: Default::default(),
		});

		assert_eq!(result, expected_embed);
		assert_eq!(data.read().await.wake_on_lan, expected_data);
	}

	#[tokio::test]
	async fn given_invalid_mac_then_returns_error_and_does_not_update_data() {
		let data = mock_data(None);

		let result = process_add_machine(
			&data,
			"NewMachine".to_string(),
			"invalid_mac".to_string(),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid MAC Address")
			.colour(Colour(0xdd2e44))
			.description("Mac address invalid_mac is invalid: Expected 6 parts in MAC address separated by `:`, but got 1");

		assert_eq!(result, expected_embed);
		assert!(data.read().await.wake_on_lan.is_empty());
	}

	#[tokio::test]
	async fn given_mac_with_invalid_hex_then_returns_error_and_does_not_update_data() {
		let data = mock_data(None);

		let result = process_add_machine(
			&data,
			"NewMachine".to_string(),
			"AA:BB:CC:DD:EE:PP".to_string(),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid MAC Address")
			.colour(Colour(0xdd2e44))
			.description("Mac address AA:BB:CC:DD:EE:PP is invalid: Invalid hexadecimal value PP");

		assert_eq!(result, expected_embed);
		assert!(data.read().await.wake_on_lan.is_empty());
	}

	#[tokio::test]
	async fn given_valid_input_then_returns_success_and_inserts_new_machine() {
		let data = mock_data(None);

		let result = process_add_machine(
			&data,
			"NewMachine".to_string(),
			"00:00:00:00:00:01".to_string(),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Success")
			.colour(Colour(0x77b255))
			.description("Successfully added new machine!")
			.field("Name", "NewMachine", true)
			.field("MAC Address", "00:00:00:00:00:01", true);

		let mut expected_data = BTreeMap::new();
		expected_data.insert("NewMachine".to_string(), WakeOnLanMachineInfo {
			mac: MacAddress([0x00, 0x00, 0x00, 0x00, 0x00, 0x01]),
			authorized_users: Default::default(),
			authorized_roles: Default::default(),
		});

		assert_eq!(result, expected_embed);
		assert_eq!(data.read().await.wake_on_lan, expected_data);
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_returns_error_and_does_not_modify_data() {
		let data = mock_data(Some(json!({
            "wake_on_lan": {
                "ExistingMachine": {
                    "mac": [1, 2, 3, 4, 5, 6],
                    "authorized_users": [],
                    "authorized_roles": []
                }
            }
        })));

		let result = process_remove_machine(
			&data,
			"NonexistentMachine".to_string(),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonexistentMachine exists");

		let mut expected_data = BTreeMap::new();
		expected_data.insert("ExistingMachine".to_string(), WakeOnLanMachineInfo {
			mac: MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
			authorized_users: Default::default(),
			authorized_roles: Default::default(),
		});

		assert_eq!(result, expected_embed);
		assert_eq!(data.read().await.wake_on_lan, expected_data);
	}

	#[tokio::test]
	async fn given_existing_machine_then_returns_success_and_removes_machine() {
		let data = mock_data(Some(json!({
            "wake_on_lan": {
                "MachineToRemove": {
                    "mac": [1, 2, 3, 4, 5, 6],
                    "authorized_users": [],
                    "authorized_roles": []
                }
            }
        })));

		let result = process_remove_machine(
			&data,
			"MachineToRemove".to_string(),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Success")
			.colour(Colour(0x77b255))
			.description("Successfully removed machine!")
			.field("Name", "MachineToRemove", true);

		let expected_data = BTreeMap::new();

		assert_eq!(result, expected_embed);
		assert_eq!(data.read().await.wake_on_lan, expected_data);
	}

	#[tokio::test]
	async fn given_no_machines_then_returns_empty_list_message() {
		let data = mock_data(Some(json!({
            "wake_on_lan": {}
        })));

		let result = process_list_machines(&data).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":information_source: Machine list")
			.colour(Colour(0x55acee))
			.description("There are no machines configured");

		assert_eq!(result, expected_embed);
	}

	#[tokio::test]
	async fn given_multiple_machines_then_returns_list_of_machines() {
		let data = mock_data(Some(json!({
            "wake_on_lan": {
                "MachineOne": {
                    "mac": [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
                    "authorized_users": [],
                    "authorized_roles": []
                },
                "MachineTwo": {
                    "mac": [0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C],
                    "authorized_users": [],
                    "authorized_roles": []
                },
                "MachineThree": {
                    "mac": [0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12],
                    "authorized_users": [],
                    "authorized_roles": []
                }
            }
        })));

		let result = process_list_machines(&data).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":information_source: Machine list")
			.colour(Colour(0x55acee))
			.description(
				"Configured machines:\n\
- MachineOne: `01:02:03:04:05:06`\n\
- MachineThree: `0D:0E:0F:10:11:12`\n\
- MachineTwo: `07:08:09:0A:0B:0C`"
			);

		assert_eq!(result, expected_embed);
	}

	#[tokio::test]
	async fn given_existing_machine_then_returns_machine_info() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"MachineOne": {
					"mac": [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
					"authorized_users": [12345678901234567u64, 12345678901234568u64],
					"authorized_roles": [98765432109876543u64, 98765432109876542u64]
				}
			}
		})));

		let result = process_describe_machine(&data, "MachineOne".to_string()).await.unwrap();

		let expected_embed = embeds::info(
			"Machine MachineOne",
			"- MAC Address: `01:02:03:04:05:06`\n\
         - Authorized Users: <@12345678901234567>, <@12345678901234568>\n\
         - Authorized Roles: <@&98765432109876542>, <@&98765432109876543>",
		);

		assert_eq!(result, expected_embed);
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_returns_error() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"MachineOne": {
					"mac": [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		let result = process_describe_machine(&data, "NonExistentMachine".to_string()).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonExistentMachine exists");

		assert_eq!(result, expected_embed);
	}

	#[tokio::test]
	async fn given_machine_with_no_authorized_users_and_roles_then_returns_info() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"MachineOne": {
					"mac": [0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		let result = process_describe_machine(&data, "MachineOne".to_string()).await.unwrap();

		let expected_embed = embeds::info(
			"Machine MachineOne",
			"- MAC Address: `01:02:03:04:05:06`\n\
         - Authorized Users: None\n\
         - Authorized Roles: None",
		);

		assert_eq!(result, expected_embed);
	}

}
