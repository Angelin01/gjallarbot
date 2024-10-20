use poise::CreateReply;
use poise::serenity_prelude::{Colour, CreateEmbed};
use crate::data::{BotData, BotError, Context};
use crate::data::wake_on_lan::WakeOnLanMachineInfo;
use crate::services::wake_on_lan::MacAddress;

#[poise::command(slash_command, owners_only, rename="add-machine")]
pub async fn add_machine(
	ctx: Context<'_>,
	#[description = "Machine name"] name: String,
	#[description = "Machine MAC Address"] mac: String,
) -> Result<(), BotError> {
	let embed = process_add_machine(ctx.data(), name, mac).await?;

	ctx.send(CreateReply::default().embed(embed)).await?;

	Ok(())
}

async fn process_add_machine(data: &BotData, name: String, mac: String) -> Result<CreateEmbed, BotError> {
	{
		let read = data.read().await;
		if read.wake_on_lan.contains_key(&name) {
			let embed = CreateEmbed::default()
				.title(":x: Duplicate name")
				.colour(Colour(0xdd2e44))
				.description(format!("A machine with name {name} already exists, try a different name"));
			return Ok(embed);
		}
	}

	let mac_address: MacAddress = match mac.parse() {
		Ok(v) => v,
		Err(e) => {
			let err_msg = e.to_string();
			let embed = CreateEmbed::default()
				.title(":x: Invalid MAC Address")
				.colour(Colour(0xdd2e44))
				.description(format!("Mac address {mac} is invalid: {err_msg}"));
			return Ok(embed);
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

	let embed = CreateEmbed::default()
		.title(":white_check_mark: Success")
		.colour(Colour(0x77b255))
		.description("Successfully added new machine!")
		.field("Name", name_field, true)
		.field("MAC Address", mac_field, true);

	Ok(embed)
}

#[poise::command(slash_command, rename="remove-machine")]
pub async fn remove_machine(
	ctx: Context<'_>,
	#[description = "Machine name"] name: String,
) -> Result<(), BotError> {
	ctx.send(CreateReply::default().ephemeral(true).content("It works")).await?;

	Ok(())
}

#[poise::command(slash_command, rename="list-machines")]
pub async fn list_machines(
	ctx: Context<'_>,
) -> Result<(), BotError> {
	ctx.send(CreateReply::default().ephemeral(true).content("It works")).await?;

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::BTreeMap;
	use serde_json::json;
	use crate::data;

	#[tokio::test]
	async fn given_duplicate_name_then_returns_error_and_does_not_update_data() {
		let data = data::tests::mock_data(Some(json!({
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
			"00:00:00:00:00:01".into()
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
		let data = data::tests::mock_data(None);

		let result = process_add_machine(
			&data,
			"NewMachine".to_string(),
			"invalid_mac".to_string()
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
		let data = data::tests::mock_data(None);

		let result = process_add_machine(
			&data,
			"NewMachine".to_string(),
			"AA:BB:CC:DD:EE:PP".to_string()
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
		let data = data::tests::mock_data(None);

		let result = process_add_machine(
			&data,
			"NewMachine".to_string(),
			"00:00:00:00:00:01".to_string()
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
}
