use super::{get_machine_info_mut, MachineError};
use crate::data::BotData;
use serenity::all::{RoleId, UserId};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub enum Entity {
	User(UserId),
	Role(RoleId),
}

#[derive(Debug, Error, PartialEq)]
pub enum AddPermissionError {
	#[error(transparent)]
	Machine(#[from] MachineError),

	#[error("{entity:?} is already permitted to wake machine {machine_name}")]
	AlreadyAuthorized {
		machine_name: String,
		entity: Entity,
	},
}

#[derive(Debug, Error, PartialEq)]
pub enum RemovePermissionError {
	#[error(transparent)]
	Machine(#[from] MachineError),

	#[error("{entity:?} is already not permitted to wake machine {machine_name}")]
	AlreadyNotAuthorized {
		machine_name: String,
		entity: Entity,
	},
}

pub async fn permit_user(
	data: &BotData,
	machine_name: &str,
	user_id: UserId,
) -> Result<(), AddPermissionError> {
	let mut lock = data.write().await;
	let mut data_write = lock.write();
	let machine_info = get_machine_info_mut(&mut data_write, machine_name).await?;

	if machine_info.authorized_users.insert(user_id) {
		Ok(())
	} else {
		Err(AddPermissionError::AlreadyAuthorized {
			machine_name: machine_name.into(),
			entity: Entity::User(user_id),
		})
	}
}

pub async fn revoke_user(
	data: &BotData,
	machine_name: &str,
	user_id: UserId,
) -> Result<(), RemovePermissionError> {
	let mut lock = data.write().await;
	let mut data_write = lock.write();

	let machine_info = get_machine_info_mut(&mut data_write, machine_name).await?;

	if machine_info.authorized_users.remove(&user_id) {
		Ok(())
	} else {
		Err(RemovePermissionError::AlreadyNotAuthorized {
			machine_name: machine_name.into(),
			entity: Entity::User(user_id),
		})
	}
}

pub async fn permit_role(
	data: &BotData,
	machine_name: &str,
	role_id: RoleId,
) -> Result<(), AddPermissionError> {
	let mut lock = data.write().await;
	let mut data_write = lock.write();

	let machine_info = get_machine_info_mut(&mut data_write, machine_name).await?;

	if machine_info.authorized_roles.insert(role_id) {
		Ok(())
	} else {
		Err(AddPermissionError::AlreadyAuthorized {
			machine_name: machine_name.into(),
			entity: Entity::Role(role_id),
		})
	}
}

pub async fn revoke_role(
	data: &BotData,
	machine_name: &str,
	role_id: RoleId,
) -> Result<(), RemovePermissionError> {
	let mut lock = data.write().await;
	let mut data_write = lock.write();

	let machine_info = get_machine_info_mut(&mut data_write, machine_name).await?;

	if machine_info.authorized_roles.remove(&role_id) {
		Ok(())
	} else {
		Err(RemovePermissionError::AlreadyNotAuthorized {
			machine_name: machine_name.into(),
			entity: Entity::Role(role_id),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::data::tests::mock_data;
	use serde_json::json;

	#[tokio::test]
	async fn given_nonexistent_machine_then_permit_user_returns_error_and_does_not_modify_data() {
		let data = mock_data(None);
		let result = permit_user(&data, "NonExistentMachine", UserId::new(12345678901234567)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::Machine(
				MachineError::NonExistentMachine {
					machine_name: "NonExistentMachine".to_string()
				}
			))
		);
		assert!(data.read().await.wake_on_lan.is_empty());
	}

	#[tokio::test]
	async fn given_already_authorized_user_then_permit_user_returns_error_and_does_not_modify_data()
	{
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [12345678901234567u64],
					"authorized_roles": []
				}
			}
		})));

		let result = permit_user(&data, "ExistingMachine", UserId::new(12345678901234567)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::AlreadyAuthorized {
				machine_name: "ExistingMachine".to_string(),
				entity: Entity::User(UserId::new(12345678901234567)),
			})
		);
		assert_eq!(
			data.read().await.wake_on_lan["ExistingMachine"]
				.authorized_users
				.len(),
			1
		);
	}

	#[tokio::test]
	async fn given_new_user_then_permit_user_returns_success_and_adds_user() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		let result = permit_user(&data, "ExistingMachine", UserId::new(12345678901234567)).await;

		assert_eq!(result, Ok(()));
		assert!(data.read().await.wake_on_lan["ExistingMachine"]
			.authorized_users
			.contains(&UserId::new(12345678901234567)));
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_revoke_user_returns_error_and_does_not_modify_data() {
		let data = mock_data(None);

		let result = revoke_user(&data, "NonExistentMachine", UserId::new(12345678901234567)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::Machine(
				MachineError::NonExistentMachine {
					machine_name: "NonExistentMachine".to_string()
				}
			))
		);
		assert!(data.read().await.wake_on_lan.is_empty());
	}

	#[tokio::test]
	async fn given_non_authorized_user_then_revoke_user_returns_error_and_does_not_modify_data() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [12345678901234567u64],
					"authorized_roles": []
				}
			}
		})));

		let result = revoke_user(&data, "ExistingMachine", UserId::new(76543210987654321)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::AlreadyNotAuthorized {
				machine_name: "ExistingMachine".to_string(),
				entity: Entity::User(UserId::new(76543210987654321))
			})
		);
		assert_eq!(
			data.read().await.wake_on_lan["ExistingMachine"]
				.authorized_users
				.len(),
			1
		);
	}

	#[tokio::test]
	async fn given_authorized_user_then_revoke_user_returns_success_and_removes_user() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [12345678901234567u64],
					"authorized_roles": []
				}
			}
		})));

		let result = revoke_user(&data, "ExistingMachine", UserId::new(12345678901234567)).await;

		assert_eq!(result, Ok(()));
		assert!(!data.read().await.wake_on_lan["ExistingMachine"]
			.authorized_users
			.contains(&UserId::new(12345678901234567)));
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_permit_role_returns_error_and_does_not_modify_data() {
		let data = mock_data(None);

		let result = permit_role(&data, "NonExistentMachine", RoleId::new(98765432109876543)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::Machine(
				MachineError::NonExistentMachine {
					machine_name: "NonExistentMachine".to_string()
				}
			))
		);
		assert!(data.read().await.wake_on_lan.is_empty());
	}

	#[tokio::test]
	async fn given_already_authorized_role_then_permit_role_returns_error_and_does_not_modify_data()
	{
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": [98765432109876543u64]
				}
			}
		})));

		let result = permit_role(&data, "ExistingMachine", RoleId::new(98765432109876543)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::AlreadyAuthorized {
				machine_name: "ExistingMachine".to_string(),
				entity: Entity::Role(RoleId::new(98765432109876543))
			})
		);
		assert_eq!(
			data.read().await.wake_on_lan["ExistingMachine"]
				.authorized_roles
				.len(),
			1
		);
	}

	#[tokio::test]
	async fn given_new_role_then_permit_role_returns_success_and_adds_role() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		let result = permit_role(&data, "ExistingMachine", RoleId::new(98765432109876543)).await;

		assert_eq!(result, Ok(()));
		assert!(data.read().await.wake_on_lan["ExistingMachine"]
			.authorized_roles
			.contains(&RoleId::new(98765432109876543)));
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_revoke_role_returns_error_and_does_not_modify_data() {
		let data = mock_data(None);

		let result = revoke_role(&data, "NonExistentMachine", RoleId::new(98765432109876543)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::Machine(
				MachineError::NonExistentMachine {
					machine_name: "NonExistentMachine".to_string()
				}
			))
		);
		assert!(data.read().await.wake_on_lan.is_empty());
	}

	#[tokio::test]
	async fn given_non_authorized_role_then_revoke_role_returns_error_and_does_not_modify_data() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": [12345678901234567u64]
				}
			}
		})));

		let result = revoke_role(&data, "ExistingMachine", RoleId::new(98765432109876543)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::AlreadyNotAuthorized {
				machine_name: "ExistingMachine".to_string(),
				entity: Entity::Role(RoleId::new(98765432109876543))
			})
		);
		assert_eq!(
			data.read().await.wake_on_lan["ExistingMachine"]
				.authorized_roles
				.len(),
			1
		);
	}

	#[tokio::test]
	async fn given_authorized_role_then_revoke_role_returns_success_and_removes_role() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": [98765432109876543u64]
				}
			}
		})));

		let result = revoke_role(&data, "ExistingMachine", RoleId::new(98765432109876543)).await;

		assert_eq!(result, Ok(()));
		assert!(!data.read().await.wake_on_lan["ExistingMachine"]
			.authorized_roles
			.contains(&RoleId::new(98765432109876543)));
	}
}
