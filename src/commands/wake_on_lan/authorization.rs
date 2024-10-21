use poise::CreateReply;
use poise::serenity_prelude::{Colour, CreateAllowedMentions, CreateEmbed, User, UserId};
use super::autocomplete_machine_name;
use crate::data::{BotData, BotError, Context};

#[poise::command(slash_command, owners_only, rename = "add-user")]
pub async fn add_user(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	machine_name: String,
	#[description = "User to authorize"] user: User,
) -> Result<(), BotError> {
	let embed = process_add_user(ctx.data(), machine_name, user.id).await?;

	ctx.send(
		CreateReply::default()
			.embed(embed)
			.allowed_mentions(CreateAllowedMentions::default().empty_users().empty_roles())
	).await?;

	Ok(())
}

async fn process_add_user(data: &BotData, machine_name: String, user_id: UserId) -> Result<CreateEmbed, BotError> {
	let mut lock = data.write().await;
	let mut data_write = lock.write();

	let machine_info = match data_write.wake_on_lan.get_mut(&machine_name) {
		Some(info) => info,
		None => {
			let embed = CreateEmbed::default()
				.title(":x: Invalid Machine")
				.colour(Colour(0xdd2e44))
				.description(format!("No machine with name {machine_name} exists"));
			return Ok(embed);
		}
	};

	if machine_info.authorized_users.contains(&user_id) {
		let embed = CreateEmbed::default()
			.title(":x: User already added")
			.colour(Colour(0xdd2e44))
			.description(format!("User <@{user_id}> is already authorized for machine {machine_name}"));
		return Ok(embed);
	}

	machine_info.authorized_users.insert(user_id);

	let embed = CreateEmbed::default()
		.title(":white_check_mark: User added")
		.colour(Colour(0x77b255))
		.description("Successfully added user to the machine!")
		.field("Machine", machine_name, true)
		.field("User", format!("<@{user_id}>"), true);

	Ok(embed)
}

#[poise::command(slash_command, owners_only, rename = "remove-user")]
pub async fn remove_user(
	ctx: Context<'_>,
	#[description = "Machine name"] name: String,
	#[description = "User that will no longer be allowed to turn this machine on"] user: User,
) -> Result<(), BotError> {
	let embed = process_remove_user(ctx.data(), name, user.id).await?;

	ctx.send(
		CreateReply::default()
			.embed(embed)
			.allowed_mentions(CreateAllowedMentions::default().empty_users().empty_roles())
	).await?;

	Ok(())
}

async fn process_remove_user(data: &BotData, machine_name: String, user_id: UserId) -> Result<CreateEmbed, BotError> {
	let mut lock = data.write().await;
	let mut data_write = lock.write();

	let machine_info = match data_write.wake_on_lan.get_mut(&machine_name) {
		Some(info) => info,
		None => {
			let embed = CreateEmbed::default()
				.title(":x: Invalid Machine")
				.colour(Colour(0xdd2e44))
				.description(format!("No machine with name {machine_name} exists"));
			return Ok(embed);
		}
	};

	if !machine_info.authorized_users.contains(&user_id) {
		let embed = CreateEmbed::default()
			.title(":x: User not found")
			.colour(Colour(0xdd2e44))
			.description(format!("User <@{user_id}> is not authorized for machine {machine_name}"));
		return Ok(embed);
	}

	machine_info.authorized_users.retain(|&id| id != user_id);

	let embed = CreateEmbed::default()
		.title(":white_check_mark: User removed")
		.colour(Colour(0x77b255))
		.description("Successfully removed user from the machine!")
		.field("Machine", machine_name, true)
		.field("User", format!("<@{user_id}>"), true);

	Ok(embed)
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;
	use crate::data::tests::mock_data;

	#[tokio::test]
	async fn given_nonexistent_machine_then_add_user_returns_error_and_does_not_modify_data() {
		let data = mock_data(None);

		let result = process_add_user(
			&data,
			"NonExistentMachine".to_string(),
			UserId::new(12345678901234567),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonExistentMachine exists");

		assert_eq!(result, expected_embed);
		assert!(data.read().await.wake_on_lan.is_empty());
	}

	#[tokio::test]
	async fn given_already_authorized_user_then_add_user_returns_error_and_does_not_modify_data() {
		let data = mock_data(Some(json!({
            "wake_on_lan": {
                "ExistingMachine": {
                    "mac": [1, 2, 3, 4, 5, 6],
                    "authorized_users": [12345678901234567u64],
                    "authorized_roles": []
                }
            }
        })));

		let result = process_add_user(
			&data,
			"ExistingMachine".to_string(),
			UserId::new(12345678901234567),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: User already added")
			.colour(Colour(0xdd2e44))
			.description("User <@12345678901234567> is already authorized for machine ExistingMachine");

		assert_eq!(result, expected_embed);
		assert_eq!(data.read().await.wake_on_lan["ExistingMachine"].authorized_users.len(), 1);
	}

	#[tokio::test]
	async fn given_new_user_then_add_user_returns_success_and_adds_user() {
		let data = mock_data(Some(json!({
            "wake_on_lan": {
                "ExistingMachine": {
                    "mac": [1, 2, 3, 4, 5, 6],
                    "authorized_users": [],
                    "authorized_roles": []
                }
            }
        })));

		let result = process_add_user(
			&data,
			"ExistingMachine".to_string(),
			UserId::new(12345678901234567),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: User added")
			.colour(Colour(0x77b255))
			.description("Successfully added user to the machine!")
			.field("Machine", "ExistingMachine", true)
			.field("User", "<@12345678901234567>", true);

		assert_eq!(result, expected_embed);
		assert!(data.read().await.wake_on_lan["ExistingMachine"].authorized_users.contains(&UserId::new(12345678901234567)));
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_remove_user_returns_error_and_does_not_modify_data() {
		let data = mock_data(None);

		let result = process_remove_user(
			&data,
			"NonExistentMachine".to_string(),
			UserId::new(12345678901234567),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonExistentMachine exists");

		assert_eq!(result, expected_embed);
		assert!(data.read().await.wake_on_lan.is_empty());
	}

	#[tokio::test]
	async fn given_non_authorized_user_then_returns_error_and_does_not_modify_data() {
		let data = mock_data(Some(json!({
            "wake_on_lan": {
                "ExistingMachine": {
                    "mac": [1, 2, 3, 4, 5, 6],
                    "authorized_users": [12345678901234567u64],
                    "authorized_roles": []
                }
            }
        })));

		let result = process_remove_user(
			&data,
			"ExistingMachine".to_string(),
			UserId::new(76543210987654321),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: User not found")
			.colour(Colour(0xdd2e44))
			.description("User <@76543210987654321> is not authorized for machine ExistingMachine");

		assert_eq!(result, expected_embed);
		assert_eq!(data.read().await.wake_on_lan["ExistingMachine"].authorized_users.len(), 1);
	}

	#[tokio::test]
	async fn given_authorized_user_then_returns_success_and_removes_user() {
		let data = mock_data(Some(json!({
            "wake_on_lan": {
                "ExistingMachine": {
                    "mac": [1, 2, 3, 4, 5, 6],
                    "authorized_users": [12345678901234567u64],
                    "authorized_roles": []
                }
            }
        })));

		let result = process_remove_user(
			&data,
			"ExistingMachine".to_string(),
			UserId::new(12345678901234567),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: User removed")
			.colour(Colour(0x77b255))
			.description("Successfully removed user from the machine!")
			.field("Machine", "ExistingMachine", true)
			.field("User", "<@12345678901234567>", true);

		assert_eq!(result, expected_embed);
		assert!(!data.read().await.wake_on_lan["ExistingMachine"].authorized_users.contains(&UserId::new(12345678901234567)));
	}
}
