use log::info;
use poise::CreateReply;
use poise::serenity_prelude::{CreateAllowedMentions, CreateEmbed, Role, RoleId, User, UserId};
use super::autocomplete_machine_name;
use crate::data::{BotData, BotError, Context};
use crate::embeds;

/// Authorizes a user to wake up a specific machine
#[poise::command(slash_command, owners_only, rename = "add-user")]
pub async fn add_user(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	machine_name: String,
	#[description = "User that be allowed wake this machine"] user: User,
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
		None => return Ok(embeds::invalid_machine(&machine_name)),
	};

	let embed = if machine_info.authorized_users.insert(user_id) {
		info!("Authorized user ID {user_id} on machine {machine_name}");
		embeds::success("User added", "Successfully added user to the machine!")
			.field("Machine", machine_name, true)
			.field("User", format!("<@{user_id}>"), true)
	}
	else {
		embeds::error(
			"User already added",
			format!("User <@{user_id}> is already authorized for machine {machine_name}"),
		)
	};

	Ok(embed)
}

/// Deauthorizes a previously configured user from waking up a specific machine
#[poise::command(slash_command, owners_only, rename = "remove-user")]
pub async fn remove_user(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	machine_name: String,
	#[description = "User that will no longer be allowed wake this machine"] user: User,
) -> Result<(), BotError> {
	let embed = process_remove_user(ctx.data(), machine_name, user.id).await?;

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
		None => return Ok(embeds::invalid_machine(&machine_name)),
	};

	let embed = if machine_info.authorized_users.remove(&user_id) {
		info!("Removed authorization from user ID {user_id} on machine {machine_name}");
		embeds::success("User removed","Successfully removed user from the machine!")
			.field("Machine", machine_name, true)
			.field("User", format!("<@{user_id}>"), true)
	}
	else {
		embeds::error(
			"User not found",
			format!("User <@{user_id}> is not authorized for machine {machine_name}"),
		)
	};

	Ok(embed)
}

/// Authorizes a role to wake up a specific machine
#[poise::command(slash_command, owners_only, rename = "add-role")]
pub async fn add_role(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	machine_name: String,
	#[description = "Role that be allowed wake this machine"] role: Role,
) -> Result<(), BotError> {
	let embed = process_add_role(ctx.data(), machine_name, role.id).await?;

	ctx.send(
		CreateReply::default()
			.embed(embed)
			.allowed_mentions(CreateAllowedMentions::default().empty_users().empty_roles())
	).await?;

	Ok(())
}

async fn process_add_role(data: &BotData, machine_name: String, role_id: RoleId) -> Result<CreateEmbed, BotError> {
	let mut lock = data.write().await;
	let mut data_write = lock.write();

	let machine_info = match data_write.wake_on_lan.get_mut(&machine_name) {
		Some(info) => info,
		None => return Ok(embeds::invalid_machine(&machine_name)),
	};

	let embed = if machine_info.authorized_roles.insert(role_id) {
		info!("Authorized role ID {role_id} on machine {machine_name}");
		embeds::success("Role added","Successfully added role to the machine!")
			.field("Machine", machine_name, true)
			.field("Role", format!("<@&{role_id}>"), true)
	}
	else {
		embeds::error(
			"Role already added",
			format!("Role <@&{role_id}> is already authorized for machine {machine_name}"),
		)
	};

	Ok(embed)
}

/// Deauthorizes a previously configured role from waking up a specific machine
#[poise::command(slash_command, owners_only, rename = "remove-role")]
pub async fn remove_role(
	ctx: Context<'_>,
	#[description = "Machine name"]
	#[autocomplete = "autocomplete_machine_name"]
	machine_name: String,
	#[description = "Role that will no longer be allowed wake this machine"] role: Role,
) -> Result<(), BotError> {
	let embed = process_remove_role(ctx.data(), machine_name, role.id).await?;

	ctx.send(
		CreateReply::default()
			.embed(embed)
			.allowed_mentions(CreateAllowedMentions::default().empty_users().empty_roles())
	).await?;

	Ok(())
}

async fn process_remove_role(data: &BotData, machine_name: String, role_id: RoleId) -> Result<CreateEmbed, BotError> {
	let mut lock = data.write().await;
	let mut data_write = lock.write();

	let machine_info = match data_write.wake_on_lan.get_mut(&machine_name) {
		Some(info) => info,
		None => return Ok(embeds::invalid_machine(&machine_name)),
	};

	let embed = if machine_info.authorized_roles.remove(&role_id) {
		info!("Removed authorization from role ID {role_id} on machine {machine_name}");
		embeds::success("Role removed","Successfully removed role from the machine!")
			.field("Machine", machine_name, true)
			.field("Role", format!("<@&{role_id}>"), true)
	}
	else {
		embeds::error(
			"Role not found",
			format!("Role <@&{role_id}> is not authorized for machine {machine_name}"),
		)
	};

	Ok(embed)
}

#[cfg(test)]
mod tests {
	use poise::serenity_prelude::Colour;
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

	#[tokio::test]
	async fn given_nonexistent_machine_then_add_role_returns_error_and_does_not_modify_data() {
		let data = mock_data(None);

		let result = process_add_role(
			&data,
			"NonExistentMachine".to_string(),
			RoleId::new(98765432109876543),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonExistentMachine exists");

		assert_eq!(result, expected_embed);
		assert!(data.read().await.wake_on_lan.is_empty());
	}

	#[tokio::test]
	async fn given_already_authorized_role_then_add_role_returns_error_and_does_not_modify_data() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": [98765432109876543u64]
				}
			}
		})));

		let result = process_add_role(
			&data,
			"ExistingMachine".to_string(),
			RoleId::new(98765432109876543),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Role already added")
			.colour(Colour(0xdd2e44))
			.description("Role <@&98765432109876543> is already authorized for machine ExistingMachine");

		assert_eq!(result, expected_embed);
		assert_eq!(data.read().await.wake_on_lan["ExistingMachine"].authorized_roles.len(), 1);
	}

	#[tokio::test]
	async fn given_new_role_then_add_role_returns_success_and_adds_role() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		let result = process_add_role(
			&data,
			"ExistingMachine".to_string(),
			RoleId::new(98765432109876543),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Role added")
			.colour(Colour(0x77b255))
			.description("Successfully added role to the machine!")
			.field("Machine", "ExistingMachine", true)
			.field("Role", "<@&98765432109876543>", true);

		assert_eq!(result, expected_embed);
		assert!(data.read().await.wake_on_lan["ExistingMachine"].authorized_roles.contains(&RoleId::new(98765432109876543)));
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_remove_role_returns_error_and_does_not_modify_data() {
		let data = mock_data(None);

		let result = process_remove_role(
			&data,
			"NonExistentMachine".to_string(),
			RoleId::new(98765432109876543),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Machine")
			.colour(Colour(0xdd2e44))
			.description("No machine with name NonExistentMachine exists");

		assert_eq!(result, expected_embed);
		assert!(data.read().await.wake_on_lan.is_empty());
	}

	#[tokio::test]
	async fn given_non_authorized_role_then_remove_role_returns_error_and_does_not_modify_data() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": [12345678901234567u64]
				}
			}
		})));

		let result = process_remove_role(
			&data,
			"ExistingMachine".to_string(),
			RoleId::new(98765432109876543),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":x: Role not found")
			.colour(Colour(0xdd2e44))
			.description("Role <@&98765432109876543> is not authorized for machine ExistingMachine");

		assert_eq!(result, expected_embed);
		assert_eq!(data.read().await.wake_on_lan["ExistingMachine"].authorized_roles.len(), 1);
	}

	#[tokio::test]
	async fn given_authorized_role_then_remove_role_returns_success_and_removes_role() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": [98765432109876543u64]
				}
			}
		})));

		let result = process_remove_role(
			&data,
			"ExistingMachine".to_string(),
			RoleId::new(98765432109876543),
		).await.unwrap();

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Role removed")
			.colour(Colour(0x77b255))
			.description("Successfully removed role from the machine!")
			.field("Machine", "ExistingMachine", true)
			.field("Role", "<@&98765432109876543>", true);

		assert_eq!(result, expected_embed);
		assert!(!data.read().await.wake_on_lan["ExistingMachine"].authorized_roles.contains(&RoleId::new(98765432109876543)));
	}
}
