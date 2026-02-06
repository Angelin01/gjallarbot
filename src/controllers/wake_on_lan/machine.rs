use super::{get_machine_info, MachineError};
use crate::data::wake_on_lan::WakeOnLanMachineInfo;
use crate::data::BotData;
use crate::db::DbConnection;
use crate::errors::{InvalidMacError, UnexpectedError};
use crate::models::wake_on_lan::{NewWakeOnLanMachine, WakeOnLanMachine};
use crate::schema::wake_on_lan_machines;
use diesel::result::DatabaseErrorKind;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use log::info;
use std::ops::AsyncFnOnce;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum AddMachineError {
	#[error(transparent)]
	Machine(#[from] MachineError),

	#[error(transparent)]
	InvalidMac(#[from] InvalidMacError),

	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

#[derive(Debug, Error, PartialEq)]
pub enum RemoveMachineError {
	#[error(transparent)]
	Machine(#[from] MachineError),

	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

#[derive(Debug, Error, PartialEq)]
pub enum ListMachinesError {
	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

pub async fn add_machine<D: DbConnection>(
	conn: &mut D,
	name: &str,
	mac: &str,
) -> Result<(), AddMachineError> {
	let mac_address = mac.parse()?;

	let new_machine = NewWakeOnLanMachine {
		name,
		mac: &mac_address,
	};

	diesel::insert_into(wake_on_lan_machines::table)
		.values(&new_machine)
		.execute(conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
				MachineError::AlreadyExists {
					machine_name: name.into(),
				}
				.into()
			}
			other => AddMachineError::Unexpected(UnexpectedError(
				anyhow::Error::new(other).context("Failed to insert machine into database"),
			)),
		})?;

	info!("Added machine {name} with MAC {mac}");
	Ok(())
}

pub async fn remove_machine<D: DbConnection>(
	conn: &mut D,
	name: &str,
) -> Result<(), RemoveMachineError> {
	let affected = diesel::delete(
		wake_on_lan_machines::table.filter(wake_on_lan_machines::name.eq(name)),
	)
	.execute(conn)
	.await
	.map_err(|e| {
		RemoveMachineError::Unexpected(UnexpectedError(
			anyhow::Error::new(e).context("Failed to delete machine from database"),
		))
	})?;

	if affected == 0 {
		return Err(MachineError::DoesNotExist {
			machine_name: name.into(),
		}
		.into());
	}

	info!("Removed machine {name}");

	Ok(())
}

pub async fn list_machines<D: DbConnection>(
	conn: &mut D,
) -> Result<Vec<WakeOnLanMachine>, ListMachinesError> {
	wake_on_lan_machines::table
		.load(conn)
		.await
		.map_err(|e| {
			ListMachinesError::Unexpected(UnexpectedError(
				anyhow::Error::new(e).context("Failed to load machines from database"),
			))
		})
}

pub trait DescribeMachineCallback<T> =
	AsyncFnOnce(Result<&WakeOnLanMachineInfo, MachineError>, &str) -> T;
pub async fn describe_machine<T, F: DescribeMachineCallback<T>>(
	data: &BotData,
	name: &str,
	func: F,
) -> T {
	let read = data.read().await;

	let machine = get_machine_info(&read, name).await;

	func.async_call_once((machine, name)).await
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::data::tests::mock_data;
	use crate::db::tests::setup_test_db;
	use crate::services::wake_on_lan::MacAddress;
	use serde_json::json;

	async fn insert_test_machine<D: DbConnection>(
		conn: &mut D,
		name: &str,
		mac: &str,
	) {
		use crate::schema::wake_on_lan_machines;

		diesel::insert_into(wake_on_lan_machines::table)
			.values(&NewWakeOnLanMachine { name, mac: &mac.parse().unwrap() })
			.execute(conn)
			.await
			.expect("Failed to insert test machine");
	}

	async fn get_all_machines<D: DbConnection>(
		conn: &mut D,
	) -> Vec<WakeOnLanMachine> {
		use crate::schema::wake_on_lan_machines;

		wake_on_lan_machines::table
			.load(conn)
			.await
			.expect("Failed to load machines")
	}

	#[tokio::test]
	async fn given_duplicate_name_then_add_machine_returns_error_and_does_not_update_data() {
		let mut conn = setup_test_db().await;

		insert_test_machine(&mut conn, "SomeMachine", "01:02:03:04:05:06").await;

		let result = add_machine(&mut conn, "SomeMachine", "00:00:00:00:00:01").await;

		assert_eq!(
			result,
			Err(AddMachineError::Machine(MachineError::AlreadyExists {
				machine_name: "SomeMachine".into(),
			}))
		);

		let machines = get_all_machines(&mut conn).await;
		assert_eq!(machines.len(), 1);
		assert_eq!(machines[0].name, "SomeMachine");
		assert_eq!(machines[0].mac, MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]));
	}

	#[tokio::test]
	async fn given_invalid_mac_then_add_machine_returns_error_and_does_not_update_data() {
		let mut conn = setup_test_db().await;

		let result = add_machine(&mut conn, "NewMachine", "invalid_mac").await;

		assert_eq!(
			result,
			Err(AddMachineError::InvalidMac(
				InvalidMacError::WrongPartCount {
					expected: 6,
					actual: 1,
				}
			))
		);

		let machines = get_all_machines(&mut conn).await;
		assert!(machines.is_empty());
	}

	#[tokio::test]
	async fn given_mac_with_invalid_hex_then_add_machine_returns_error_and_does_not_update_data() {
		let mut conn = setup_test_db().await;

		let result = add_machine(&mut conn, "NewMachine", "AA:BB:CC:DD:EE:PP").await;

		assert_eq!(
			result,
			Err(AddMachineError::InvalidMac(
				InvalidMacError::InvalidHexString("PP".into())
			))
		);

		let machines = get_all_machines(&mut conn).await;
		assert!(machines.is_empty());
	}

	#[tokio::test]
	async fn given_valid_input_then_add_machine_returns_success_and_inserts_new_machine() {
		let mut conn = setup_test_db().await;

		let result = add_machine(&mut conn, "NewMachine", "00:00:00:00:00:01").await;

		assert_eq!(result, Ok(()));

		let machines = get_all_machines(&mut conn).await;
		assert_eq!(machines.len(), 1);
		assert_eq!(machines[0].name, "NewMachine");
		assert_eq!(machines[0].mac, MacAddress([0x00, 0x00, 0x00, 0x00, 0x00, 0x01]));
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_remove_machine_returns_error_and_does_not_modify_data()
	{
		let mut conn = setup_test_db().await;

		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		let result = remove_machine(&mut conn, "NonexistentMachine").await;

		assert_eq!(
			result,
			Err(RemoveMachineError::Machine(MachineError::DoesNotExist {
				machine_name: "NonexistentMachine".into(),
			}))
		);

		let machines = get_all_machines(&mut conn).await;
		assert_eq!(machines.len(), 1);
		assert_eq!(machines[0].name, "ExistingMachine");
	}

	#[tokio::test]
	async fn given_existing_machine_then_remove_machine_returns_success_and_removes_machine() {
		let mut conn = setup_test_db().await;

		insert_test_machine(&mut conn, "MachineToRemove", "01:02:03:04:05:06").await;

		let result = remove_machine(&mut conn, "MachineToRemove").await;

		assert_eq!(result, Ok(()));

		let machines = get_all_machines(&mut conn).await;
		assert!(machines.is_empty());
	}

	#[tokio::test]
	async fn given_no_machines_then_list_machines_returns_empty_vec() {
		let mut conn = setup_test_db().await;

		let result = list_machines(&mut conn).await.unwrap();

		assert!(result.is_empty());
	}

	#[tokio::test]
	async fn given_machines_exist_then_list_machines_returns_all_machines() {
		let mut conn = setup_test_db().await;

		insert_test_machine(&mut conn, "MachineA", "01:02:03:04:05:06").await;
		insert_test_machine(&mut conn, "MachineB", "07:08:09:0A:0B:0C").await;

		let result = list_machines(&mut conn).await.unwrap();

		assert_eq!(result.len(), 2);
		assert_eq!(result[0].name, "MachineA");
		assert_eq!(result[0].mac, MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]));
		assert_eq!(result[1].name, "MachineB");
		assert_eq!(result[1].mac, MacAddress([0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C]));
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_describe_machine_callbacks_with_error() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		describe_machine(&data, "NonExistentMachine", async |result, name| {
			assert_eq!(name, "NonExistentMachine");
			assert_eq!(
				result,
				Err(MachineError::DoesNotExist {
					machine_name: "NonExistentMachine".into(),
				})
			);
		})
		.await;
	}

	#[tokio::test]
	async fn given_existing_machine_then_describe_machine_calls_function_with_machine_info() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": []
				}
			}
		})));

		describe_machine(&data, "ExistingMachine", async |result, name| {
			assert_eq!(name, "ExistingMachine");
			match result {
				Ok(machine) => assert_eq!(
					machine.mac,
					MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06])
				),
				Err(_) => assert!(false, "received error when it was not expected"),
			}
		})
		.await;
	}
}
