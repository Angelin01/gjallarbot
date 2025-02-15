use super::{get_machine_info, MachineError};
use crate::data::wake_on_lan::WakeOnLanMachineInfo;
use crate::data::BotData;
use crate::errors::InvalidMacError;
use crate::services::wake_on_lan::MacAddress;
use log::info;
use std::ops::AsyncFnOnce;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum AddMachineError {
	#[error(transparent)]
	Machine(#[from] MachineError),

	#[error(transparent)]
	InvalidMac(#[from] InvalidMacError),
}

#[derive(Debug, Error, PartialEq)]
pub enum RemoveMachineError {
	#[error(transparent)]
	Machine(#[from] MachineError),
}

pub async fn add_machine(data: &BotData, name: &str, mac: &str) -> Result<(), AddMachineError> {
	{
		let read = data.read().await;
		if read.wake_on_lan.contains_key(name) {
			return Err(MachineError::AlreadyExists {
				machine_name: name.into(),
			})?;
		}
	}

	let mac_address = mac.parse()?;

	{
		let mut lock = data.write().await;
		let mut data_write = lock.write();
		data_write.wake_on_lan.insert(
			name.into(),
			WakeOnLanMachineInfo {
				mac: mac_address,
				authorized_users: Default::default(),
				authorized_roles: Default::default(),
			},
		);
	}

	info!("Added machine {name} with MAC {mac}");

	Ok(())
}

pub async fn remove_machine(data: &BotData, name: &str) -> Result<(), RemoveMachineError> {
	{
		let read = data.read().await;
		if !read.wake_on_lan.contains_key(name) {
			return Err(MachineError::DoesNotExist {
				machine_name: name.into(),
			})?;
		}
	}

	{
		let mut lock = data.write().await;
		let mut data_write = lock.write();
		data_write.wake_on_lan.remove(name);
	}

	info!("Removed machine {name}");

	Ok(())
}

pub async fn describe_machine<F: AsyncFnOnce(&WakeOnLanMachineInfo) -> ()>(
	data: &BotData,
	name: &str,
	func: F,
) -> Result<(), MachineError> {
	let read = data.read().await;

	let machine = get_machine_info(&read, name).await?;

	func.async_call_once((machine,)).await;

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::data::tests::mock_data;
	use serde_json::json;
	use std::collections::BTreeMap;

	#[tokio::test]
	async fn given_duplicate_name_then_add_machine_returns_error_and_does_not_update_data() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"SomeMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		let result = add_machine(&data, "SomeMachine".into(), "00:00:00:00:00:01".into()).await;

		let mut expected_data = BTreeMap::new();
		expected_data.insert(
			"SomeMachine".to_string(),
			WakeOnLanMachineInfo {
				mac: MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
				authorized_users: Default::default(),
				authorized_roles: Default::default(),
			},
		);

		assert_eq!(
			result,
			Err(AddMachineError::Machine(MachineError::AlreadyExists {
				machine_name: "SomeMachine".into(),
			}))
		);
		assert_eq!(data.read().await.wake_on_lan, expected_data);
	}

	#[tokio::test]
	async fn given_invalid_mac_then_add_machine_returns_error_and_does_not_update_data() {
		let data = mock_data(None);

		let result = add_machine(&data, "NewMachine", "invalid_mac").await;

		assert_eq!(
			result,
			Err(AddMachineError::InvalidMac(
				InvalidMacError::WrongPartCount {
					expected: 6,
					actual: 1,
				}
			))
		);
		assert!(data.read().await.wake_on_lan.is_empty());
	}

	#[tokio::test]
	async fn given_mac_with_invalid_hex_then_add_machine_returns_error_and_does_not_update_data() {
		let data = mock_data(None);

		let result = add_machine(&data, "NewMachine", "AA:BB:CC:DD:EE:PP").await;

		assert_eq!(
			result,
			Err(AddMachineError::InvalidMac(
				InvalidMacError::InvalidHexString("PP".into())
			))
		);
		assert!(data.read().await.wake_on_lan.is_empty());
	}

	#[tokio::test]
	async fn given_valid_input_then_add_machine_returns_success_and_inserts_new_machine() {
		let data = mock_data(None);

		let result = add_machine(&data, "NewMachine", "00:00:00:00:00:01").await;

		let mut expected_data = BTreeMap::new();
		expected_data.insert(
			"NewMachine".to_string(),
			WakeOnLanMachineInfo {
				mac: MacAddress([0x00, 0x00, 0x00, 0x00, 0x00, 0x01]),
				authorized_users: Default::default(),
				authorized_roles: Default::default(),
			},
		);

		assert_eq!(result, Ok(()));
		assert_eq!(data.read().await.wake_on_lan, expected_data);
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_remove_machine_returns_error_and_does_not_modify_data()
	{
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		let result = remove_machine(&data, "NonexistentMachine").await;

		let mut expected_data = BTreeMap::new();
		expected_data.insert(
			"ExistingMachine".to_string(),
			WakeOnLanMachineInfo {
				mac: MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
				authorized_users: Default::default(),
				authorized_roles: Default::default(),
			},
		);

		assert_eq!(
			result,
			Err(RemoveMachineError::Machine(MachineError::DoesNotExist {
				machine_name: "NonexistentMachine".into(),
			}))
		);
		assert_eq!(data.read().await.wake_on_lan, expected_data);
	}

	#[tokio::test]
	async fn given_existing_machine_then_remove_machine_returns_success_and_removes_machine() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"MachineToRemove": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		let result = remove_machine(&data, "MachineToRemove").await;

		let expected_data = BTreeMap::new();

		assert_eq!(result, Ok(()));
		assert_eq!(data.read().await.wake_on_lan, expected_data);
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_describe_machine_returns_error() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		let result = describe_machine(&data, "NonexistentMachine", async |_| {}).await;

		assert_eq!(
			result,
			Err(MachineError::DoesNotExist {
				machine_name: "NonexistentMachine".into(),
			})
		);
	}

	#[tokio::test]
	async fn given_existing_machine_then_describe_machine_returns_success_and_calls_function() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		let mut called = false;

		let result = describe_machine(&data, "ExistingMachine", async |machine| {
			assert_eq!(
				machine.mac,
				MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06])
			);
			called = true;
		})
		.await;

		assert!(called);
		assert_eq!(result, Ok(()));
	}
}
