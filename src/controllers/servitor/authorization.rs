use super::super::DiscordEntity;
use super::{get_server_info_mut, ServerError};
use crate::data::BotData;
use log::info;
use serenity::all::UserId;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum AddPermissionError {
	#[error(transparent)]
	Server(#[from] ServerError),

	#[error("{entity:?} is already permitted to operate server {server_name}")]
	AlreadyAuthorized {
		server_name: String,
		entity: DiscordEntity,
	},
}

#[derive(Debug, Error, PartialEq)]
pub enum RemovePermissionError {
	#[error(transparent)]
	Server(#[from] ServerError),

	#[error("{entity:?} is already not permitted to operate server {server_name}")]
	AlreadyNotAuthorized {
		server_name: String,
		entity: DiscordEntity,
	},
}

pub async fn permit_user(
	data: &BotData,
	server_name: &str,
	user_id: UserId,
) -> Result<(), AddPermissionError> {
	let mut lock = data.write().await;
	let mut data_write = lock.write();

	let server_info = get_server_info_mut(&mut data_write, server_name).await?;

	if server_info.authorized_users.insert(user_id) {
		info!("Permitted user {user_id} to operate server {server_name}");
		Ok(())
	} else {
		Err(AddPermissionError::AlreadyAuthorized {
			server_name: server_name.into(),
			entity: DiscordEntity::User(user_id),
		})
	}
}

pub async fn revoke_user(
	data: &BotData,
	server_name: &str,
	user_id: UserId,
) -> Result<(), RemovePermissionError> {
	let mut lock = data.write().await;
	let mut data_write = lock.write();

	let server_info = get_server_info_mut(&mut data_write, server_name).await?;

	if server_info.authorized_users.remove(&user_id) {
		info!("Revoked user {user_id}'s permission to operate server {server_name}");
		Ok(())
	} else {
		Err(RemovePermissionError::AlreadyNotAuthorized {
			server_name: server_name.to_string(),
			entity: DiscordEntity::User(user_id),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::data::tests::mock_data;
	use serde_json::json;

	#[tokio::test]
	async fn given_non_existing_server_then_permit_user_returns_error_and_does_not_modify_data() {
		let data = mock_data(None);

		let result = permit_user(&data, "NonExistentServer", UserId::new(12345678901234567)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::Server(ServerError::DoesNotExist {
				server_name: "NonExistentServer".to_string()
			}))
		);
		assert!(data.read().await.servitor.is_empty())
	}

	#[tokio::test]
	async fn given_already_authorized_user_then_permit_user_returns_error_and_does_not_modify_data()
	{
		let data = mock_data(Some(json!({
			"servitor": {
				"ExistingServer": {
					"servitor": "foo",
					"unit_name": "bar",
					"authorized_users": [12345678901234567u64],
					"authorized_roles": []
				}
			}
		})));

		let result = permit_user(&data, "ExistingServer", UserId::new(12345678901234567)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::AlreadyAuthorized {
				server_name: "ExistingServer".to_string(),
				entity: DiscordEntity::User(UserId::new(12345678901234567u64))
			})
		);
		assert_eq!(
			data.read().await.servitor["ExistingServer"]
				.authorized_users
				.len(),
			1
		);
	}

	#[tokio::test]
	async fn given_new_user_then_permit_user_returns_success_and_adds_user() {
		let data = mock_data(Some(json!({
			"servitor": {
				"ExistingServer": {
					"servitor": "foo",
					"unit_name": "bar",
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		let result = permit_user(&data, "ExistingServer", UserId::new(12345678901234567)).await;

		assert_eq!(result, Ok(()));

		assert!(data.read().await.servitor["ExistingServer"]
			.authorized_users
			.contains(&UserId::new(12345678901234567)));
	}

	#[tokio::test]
	async fn given_non_existing_server_then_revoke_user_returns_error_and_does_not_modify_data() {
		let data = mock_data(None);

		let result = revoke_user(&data, "NonExistentServer", UserId::new(12345678901234567)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::Server(ServerError::DoesNotExist {
				server_name: "NonExistentServer".to_string()
			}))
		);
		assert!(data.read().await.servitor.is_empty());
	}

	#[tokio::test]
	async fn given_not_authorized_user_then_revoke_user_returns_error_and_does_not_modify_data() {
		let data = mock_data(Some(json!({
			"servitor": {
				"ExistingServer": {
					"servitor": "foo",
					"unit_name": "bar",
					"authorized_users": [12345678901234567u64],
					"authorized_roles": []
				}
			}
		})));

		let result = revoke_user(&data, "ExistingServer", UserId::new(76543210987654321)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::AlreadyNotAuthorized {
				server_name: "ExistingServer".to_string(),
				entity: DiscordEntity::User(UserId::new(76543210987654321))
			})
		);
		assert_eq!(
			data.read().await.servitor["ExistingServer"]
				.authorized_users
				.len(),
			1
		);
	}

	#[tokio::test]
	async fn given_authorized_user_then_revoke_user_returns_success_and_removes_users() {
		let data = mock_data(Some(json!({
			"servitor": {
				"ExistingServer": {
					"servitor": "foo",
					"unit_name": "bar",
					"authorized_users": [12345678901234567u64],
					"authorized_roles": []
				}
			}
		})));

		let result = revoke_user(&data, "ExistingServer", UserId::new(12345678901234567)).await;

		assert_eq!(result, Ok(()));
		assert!(!data.read().await.servitor["ExistingServer"]
			.authorized_users
			.contains(&UserId::new(12345678901234567)));
	}
}
