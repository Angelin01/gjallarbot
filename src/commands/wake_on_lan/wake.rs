use log::info;
use poise::CreateReply;
use poise::serenity_prelude::{CreateEmbed, User};
use super::autocomplete_machine_name;
use crate::data::{BotData, BotError, Context};
use crate::embeds;
use crate::services::wake_on_lan::MagicPacket;

// Send a machine a wake up magic packet
#[poise::command(slash_command)]
pub async fn wake(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	name: String,
) -> Result<(), BotError> {
	let (embed, magic_packet) = process_wake(ctx.data(), ctx.author(), name).await?;

	if let Some(p) = magic_packet {
		p.send().await?;
	}

	ctx.send(CreateReply::default().embed(embed)).await?;

	Ok(())
}

async fn process_wake(
	data: &BotData,
	author: &User,
	machine_name: String,
) -> Result<(CreateEmbed, Option<MagicPacket>), BotError> {
	let mut lock = data.write().await;
	let mut data_write = lock.write();

	let machine_info = match data_write.wake_on_lan.get_mut(&machine_name) {
		Some(info) => info,
		None => return Ok((embeds::error("Invalid Machine", format!("No machine with name {machine_name} exists")), None)),
	};

	let mut authorized =  machine_info.authorized_users.contains(&author.id);
	if !authorized && let Some(member) = &author.member {
		authorized = member.roles.iter().any(|&role| machine_info.authorized_roles.contains(&role));
	}

	let result = if authorized {
		let mac = &machine_info.mac;
		info!("Waking up machine {machine_name} with mac {mac}");
		(
			embeds::success("Machine woken", format!("Machine {machine_name} woken")),
			Some(MagicPacket::from_mac(mac)),
		)
	}
	else {
		(
			embeds::error("Unauthorized", format!("You are not authorized to wake machine {machine_name}")),
			None
		)
	};

	Ok(result)
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;
	use poise::serenity_prelude::{Colour, Member, RoleId, UserId};
	use crate::data::tests::mock_data;
	use crate::services::wake_on_lan::MacAddress;

	fn mock_author_dms(id: UserId) -> User {
		let mut user = User::default();
		user.id = id;
		user.name = "mock_author".to_string();
		user
	}

	fn mock_author_guild(id: UserId, roles: Vec<RoleId>) -> User {
		let mut user = mock_author_dms(id);

		let mut member = Member::default();
		member.roles = roles;

		user.member = Some(Box::new(member.into()));

		user
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_wake_returns_error() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6]
				}
			}
		})));

		let (embed, magic_packet) = process_wake(
			&data,
			&mock_author_dms(UserId::new(12345678901234567)),
			"NonexistentMachine".to_string(),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonexistentMachine exists");

		assert_eq!(embed, expected_embed);
		assert_eq!(magic_packet, None);
	}

	#[tokio::test]
	async fn given_dm_call_and_existing_machine_but_user_not_in_allowed_list_then_wake_returns_error() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [98765432109876543i64, 98765432109876542i64],
					"authorized_roles": [98765432109876541i64, 98765432109876540i64]
				}
			}
		})));

		let (embed, magic_packet) = process_wake(
			&data,
			&mock_author_dms(UserId::new(12345678901234567)),
			"ExistingMachine".to_string(),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Unauthorized")
			.colour(Colour(0xdd2e44))
			.description("You are not authorized to wake machine ExistingMachine");

		assert_eq!(embed, expected_embed);
		assert_eq!(magic_packet, None);
	}

	#[tokio::test]
	async fn given_guild_call_and_existing_machine_but_user_not_in_allowed_list_then_wake_returns_error() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [98765432109876543i64, 98765432109876542i64],
					"authorized_roles": [98765432109876541i64, 98765432109876540i64]
				}
			}
		})));

		let (embed, magic_packet) = process_wake(
			&data,
			&mock_author_guild(UserId::new(12345678901234567), vec![RoleId::new(12345678901234567)]),
			"ExistingMachine".to_string(),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Unauthorized")
			.colour(Colour(0xdd2e44))
			.description("You are not authorized to wake machine ExistingMachine");

		assert_eq!(embed, expected_embed);
		assert_eq!(magic_packet, None);
	}

	#[tokio::test]
	async fn given_dm_call_and_existing_machine_and_user_in_allowed_list_then_should_wake_machine() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [12345678901234567i64]
				}
			}
		})));

		let (embed, magic_packet) = process_wake(
			&data,
			&mock_author_dms(UserId::new(12345678901234567)),
			"ExistingMachine".to_string(),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Machine woken")
			.colour(Colour(0x77b255))
			.description("Machine ExistingMachine woken");

		let expected_magic_packet = MagicPacket::from_mac(
			&MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06])
		);

		assert_eq!(embed, expected_embed);
		assert_eq!(magic_packet, Some(expected_magic_packet));
	}

	#[tokio::test]
	async fn given_guild_call_and_existing_machine_and_user_in_allowed_list_then_should_wake_machine() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [12345678901234567i64],
					"authorized_roles": []
				}
			}
		})));

		let (embed, magic_packet) = process_wake(
			&data,
			&mock_author_guild(UserId::new(12345678901234567), vec![RoleId::new(98765432109876543)]),
			"ExistingMachine".to_string(),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Machine woken")
			.colour(Colour(0x77b255))
			.description("Machine ExistingMachine woken");

		let expected_magic_packet = MagicPacket::from_mac(
			&MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06])
		);

		assert_eq!(embed, expected_embed);
		assert_eq!(magic_packet, Some(expected_magic_packet));
	}

	#[tokio::test]
	async fn given_guild_call_and_existing_machine_and_user_in_allowed_roles_then_should_wake_machine() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": [98765432109876543i64]
				}
			}
		})));

		let (embed, magic_packet) = process_wake(
			&data,
			&mock_author_guild(UserId::new(12345678901234567), vec![RoleId::new(98765432109876543)]),
			"ExistingMachine".to_string(),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Machine woken")
			.colour(Colour(0x77b255))
			.description("Machine ExistingMachine woken");

		let expected_magic_packet = MagicPacket::from_mac(
			&MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06])
		);

		assert_eq!(embed, expected_embed);
		assert_eq!(magic_packet, Some(expected_magic_packet));
	}
}
