use super::MachineError;
use crate::db::DbConnection;
use crate::errors::{InvalidMacError, UnexpectedError};
use crate::models::wake_on_lan::{NewWakeOnLanMachine, WakeOnLanMachine};
use crate::schema::{
	wake_on_lan_machines, wake_on_lan_machines_authorized_roles,
	wake_on_lan_machines_authorized_users,
};
use diesel::result::DatabaseErrorKind;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use log::info;
use poise::serenity_prelude::{RoleId, UserId};
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

#[derive(Debug, Error, PartialEq)]
pub enum DescribeMachineError {
	#[error(transparent)]
	Machine(#[from] MachineError),

	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

#[derive(Debug)]
pub struct MachineDescription {
	pub machine: WakeOnLanMachine,
	pub authorized_users: Vec<UserId>,
	pub authorized_roles: Vec<RoleId>,
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
	let affected =
		diesel::delete(wake_on_lan_machines::table.filter(wake_on_lan_machines::name.eq(name)))
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
	wake_on_lan_machines::table.load(conn).await.map_err(|e| {
		ListMachinesError::Unexpected(UnexpectedError(
			anyhow::Error::new(e).context("Failed to load machines from database"),
		))
	})
}

pub async fn describe_machine<D: DbConnection>(
	conn: &mut D,
	name: &str,
) -> Result<MachineDescription, DescribeMachineError> {
	let machine: WakeOnLanMachine = wake_on_lan_machines::table
		.filter(wake_on_lan_machines::name.eq(name))
		.first(conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::NotFound => MachineError::DoesNotExist {
				machine_name: name.into(),
			}
			.into(),
			other => DescribeMachineError::Unexpected(UnexpectedError(
				anyhow::Error::new(other).context("Failed to query machine from database"),
			)),
		})?;

	let authorized_users: Vec<i64> = wake_on_lan_machines_authorized_users::table
		.select(wake_on_lan_machines_authorized_users::user_id)
		.filter(wake_on_lan_machines_authorized_users::machine_id.eq(machine.id))
		.load(conn)
		.await
		.map_err(|e| {
			DescribeMachineError::Unexpected(UnexpectedError(
				anyhow::Error::new(e).context("Failed to query authorized users from database"),
			))
		})?;

	let authorized_roles: Vec<i64> = wake_on_lan_machines_authorized_roles::table
		.select(wake_on_lan_machines_authorized_roles::role_id)
		.filter(wake_on_lan_machines_authorized_roles::machine_id.eq(machine.id))
		.load(conn)
		.await
		.map_err(|e| {
			DescribeMachineError::Unexpected(UnexpectedError(
				anyhow::Error::new(e).context("Failed to query authorized roles from database"),
			))
		})?;

	Ok(MachineDescription {
		machine,
		authorized_users: authorized_users
			.into_iter()
			.map(|u| UserId::new(u as u64))
			.collect(),
		authorized_roles: authorized_roles
			.into_iter()
			.map(|r| RoleId::new(r as u64))
			.collect(),
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::controllers::wake_on_lan::tests::insert_test_machine;
	use crate::db::tests::setup_test_db;
	use crate::models::wake_on_lan::{
		NewWakeOnLanMachineAuthorizedRole, NewWakeOnLanMachineAuthorizedUser,
	};
	use crate::services::wake_on_lan::MacAddress;

	async fn get_all_machines<D: DbConnection>(conn: &mut D) -> Vec<WakeOnLanMachine> {
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
		assert_eq!(
			machines[0].mac,
			MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06])
		);
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
		assert_eq!(
			machines[0].mac,
			MacAddress([0x00, 0x00, 0x00, 0x00, 0x00, 0x01])
		);
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
		assert_eq!(
			result[0].mac,
			MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06])
		);
		assert_eq!(result[1].name, "MachineB");
		assert_eq!(
			result[1].mac,
			MacAddress([0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C])
		);
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_describe_machine_returns_error() {
		let mut conn = setup_test_db().await;

		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		let result = describe_machine(&mut conn, "NonExistentMachine").await;

		assert_eq!(
			result.unwrap_err(),
			DescribeMachineError::Machine(MachineError::DoesNotExist {
				machine_name: "NonExistentMachine".into(),
			})
		);
	}

	#[tokio::test]
	async fn given_existing_machine_then_describe_machine_returns_machine_info() {
		let mut conn = setup_test_db().await;

		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		let result = describe_machine(&mut conn, "ExistingMachine")
			.await
			.unwrap();

		assert_eq!(result.machine.name, "ExistingMachine");
		assert_eq!(
			result.machine.mac,
			MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06])
		);
		assert!(result.authorized_users.is_empty());
		assert!(result.authorized_roles.is_empty());
	}

	#[tokio::test]
	async fn given_machine_with_authorized_users_and_roles_then_describe_machine_returns_them() {
		let mut conn = setup_test_db().await;

		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		let machine: WakeOnLanMachine = wake_on_lan_machines::table
			.filter(wake_on_lan_machines::name.eq("ExistingMachine"))
			.first(&mut conn)
			.await
			.unwrap();

		diesel::insert_into(wake_on_lan_machines_authorized_users::table)
			.values(&NewWakeOnLanMachineAuthorizedUser {
				machine_id: machine.id,
				user_id: 12345678901234567,
			})
			.execute(&mut conn)
			.await
			.unwrap();

		diesel::insert_into(wake_on_lan_machines_authorized_roles::table)
			.values(&NewWakeOnLanMachineAuthorizedRole {
				machine_id: machine.id,
				role_id: 98765432109876543,
			})
			.execute(&mut conn)
			.await
			.unwrap();

		let result = describe_machine(&mut conn, "ExistingMachine")
			.await
			.unwrap();

		assert_eq!(result.machine.name, "ExistingMachine");
		assert_eq!(
			result.authorized_users,
			vec![UserId::new(12345678901234567)]
		);
		assert_eq!(
			result.authorized_roles,
			vec![RoleId::new(98765432109876543)]
		);
	}
}
