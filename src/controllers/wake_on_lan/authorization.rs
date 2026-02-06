use super::MachineError;
use crate::controllers::DiscordEntity;
use crate::db::DbConnection;
use crate::errors::UnexpectedError;
use crate::models::wake_on_lan::NewWakeOnLanMachineAuthorizedUser;
use crate::schema::{wake_on_lan_machines, wake_on_lan_machines_authorized_users};
use diesel::result::DatabaseErrorKind;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use log::info;
use serenity::all::{RoleId, UserId};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum AddPermissionError {
	#[error(transparent)]
	Machine(#[from] MachineError),

	#[error("{entity:?} is already permitted to wake machine {machine_name}")]
	AlreadyAuthorized {
		machine_name: String,
		entity: DiscordEntity,
	},

	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

#[derive(Debug, Error, PartialEq)]
pub enum RemovePermissionError {
	#[error(transparent)]
	Machine(#[from] MachineError),

	#[error("{entity:?} is already not permitted to wake machine {machine_name}")]
	AlreadyNotAuthorized {
		machine_name: String,
		entity: DiscordEntity,
	},

	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

async fn get_machine_id<D: DbConnection>(
	conn: &mut D,
	machine_name: &str,
) -> Result<i32, MachineError> {
	wake_on_lan_machines::table
		.filter(wake_on_lan_machines::name.eq(machine_name))
		.select(wake_on_lan_machines::id)
		.first::<i32>(conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::NotFound => MachineError::DoesNotExist {
				machine_name: machine_name.into(),
			},
			other => panic!("Unexpected error looking up machine: {other}"),
		})
}

pub async fn permit_user<D: DbConnection>(
	conn: &mut D,
	machine_name: &str,
	user_id: UserId,
) -> Result<(), AddPermissionError> {
	let machine_id = get_machine_id(conn, machine_name).await?;

	diesel::insert_into(wake_on_lan_machines_authorized_users::table)
		.values(&NewWakeOnLanMachineAuthorizedUser {
			machine_id,
			user_id: user_id.get() as i64,
		})
		.execute(conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
				AddPermissionError::AlreadyAuthorized {
					machine_name: machine_name.into(),
					entity: DiscordEntity::User(user_id),
				}
			}
			other => AddPermissionError::Unexpected(UnexpectedError(
				anyhow::Error::new(other)
					.context("Failed to insert authorized user into database"),
			)),
		})?;

	info!("Permitted user {user_id} to wake machine {machine_name}");
	Ok(())
}

pub async fn revoke_user<D: DbConnection>(
	conn: &mut D,
	machine_name: &str,
	user_id: UserId,
) -> Result<(), RemovePermissionError> {
	let machine_id = get_machine_id(conn, machine_name).await.map_err(|e| {
		RemovePermissionError::Machine(e)
	})?;

	let affected = diesel::delete(
		wake_on_lan_machines_authorized_users::table
			.filter(wake_on_lan_machines_authorized_users::machine_id.eq(machine_id))
			.filter(wake_on_lan_machines_authorized_users::user_id.eq(user_id.get() as i64)),
	)
	.execute(conn)
	.await
	.map_err(|e| {
		RemovePermissionError::Unexpected(UnexpectedError(
			anyhow::Error::new(e)
				.context("Failed to delete authorized user from database"),
		))
	})?;

	if affected == 0 {
		return Err(RemovePermissionError::AlreadyNotAuthorized {
			machine_name: machine_name.into(),
			entity: DiscordEntity::User(user_id),
		});
	}

	info!("Revoked user's {user_id} permission to wake machine {machine_name}");
	Ok(())
}

pub async fn permit_role<D: DbConnection>(
	conn: &mut D,
	machine_name: &str,
	role_id: RoleId,
) -> Result<(), AddPermissionError> {
	use crate::models::wake_on_lan::NewWakeOnLanMachineAuthorizedRole;
	use crate::schema::wake_on_lan_machines_authorized_roles;

	let machine_id = get_machine_id(conn, machine_name).await?;

	diesel::insert_into(wake_on_lan_machines_authorized_roles::table)
		.values(&NewWakeOnLanMachineAuthorizedRole {
			machine_id,
			role_id: role_id.get() as i64,
		})
		.execute(conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
				AddPermissionError::AlreadyAuthorized {
					machine_name: machine_name.into(),
					entity: DiscordEntity::Role(role_id),
				}
			}
			other => AddPermissionError::Unexpected(UnexpectedError(
				anyhow::Error::new(other)
					.context("Failed to insert authorized role into database"),
			)),
		})?;

	info!("Permitted role {role_id} to wake machine {machine_name}");
	Ok(())
}

pub async fn revoke_role<D: DbConnection>(
	conn: &mut D,
	machine_name: &str,
	role_id: RoleId,
) -> Result<(), RemovePermissionError> {
	use crate::schema::wake_on_lan_machines_authorized_roles;

	let machine_id = get_machine_id(conn, machine_name).await.map_err(|e| {
		RemovePermissionError::Machine(e)
	})?;

	let affected = diesel::delete(
		wake_on_lan_machines_authorized_roles::table
			.filter(wake_on_lan_machines_authorized_roles::machine_id.eq(machine_id))
			.filter(wake_on_lan_machines_authorized_roles::role_id.eq(role_id.get() as i64)),
	)
	.execute(conn)
	.await
	.map_err(|e| {
		RemovePermissionError::Unexpected(UnexpectedError(
			anyhow::Error::new(e)
				.context("Failed to delete authorized role from database"),
		))
	})?;

	if affected == 0 {
		return Err(RemovePermissionError::AlreadyNotAuthorized {
			machine_name: machine_name.into(),
			entity: DiscordEntity::Role(role_id),
		});
	}

	info!("Revoked role {role_id}'s permission to wake machine {machine_name}");
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::db::tests::setup_test_db;
	use crate::models::wake_on_lan::NewWakeOnLanMachine;

	async fn insert_test_machine<D: DbConnection>(conn: &mut D, name: &str, mac: &str) {
		use crate::services::wake_on_lan::MacAddress;

		diesel::insert_into(wake_on_lan_machines::table)
			.values(&NewWakeOnLanMachine {
				name,
				mac: &mac.parse::<MacAddress>().unwrap(),
			})
			.execute(conn)
			.await
			.expect("Failed to insert test machine");
	}

	// --- permit_user tests ---

	#[tokio::test]
	async fn given_nonexistent_machine_then_permit_user_returns_machine_error() {
		let mut conn = setup_test_db().await;

		let result = permit_user(&mut conn, "NonExistentMachine", UserId::new(12345678901234567)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::Machine(MachineError::DoesNotExist {
				machine_name: "NonExistentMachine".to_string()
			}))
		);
	}

	#[tokio::test]
	async fn given_already_authorized_user_then_permit_user_returns_already_authorized() {
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		// First permit succeeds
		permit_user(&mut conn, "ExistingMachine", UserId::new(12345678901234567))
			.await
			.unwrap();

		// Second permit should fail
		let result = permit_user(&mut conn, "ExistingMachine", UserId::new(12345678901234567)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::AlreadyAuthorized {
				machine_name: "ExistingMachine".to_string(),
				entity: DiscordEntity::User(UserId::new(12345678901234567)),
			})
		);
	}

	#[tokio::test]
	async fn given_new_user_then_permit_user_returns_success() {
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		let result = permit_user(&mut conn, "ExistingMachine", UserId::new(12345678901234567)).await;

		assert_eq!(result, Ok(()));

		// Verify the user was actually inserted
		let count: i64 = wake_on_lan_machines_authorized_users::table
			.count()
			.get_result(&mut conn)
			.await
			.unwrap();
		assert_eq!(count, 1);
	}

	// --- revoke_user tests ---

	#[tokio::test]
	async fn given_nonexistent_machine_then_revoke_user_returns_machine_error() {
		let mut conn = setup_test_db().await;

		let result = revoke_user(&mut conn, "NonExistentMachine", UserId::new(12345678901234567)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::Machine(MachineError::DoesNotExist {
				machine_name: "NonExistentMachine".to_string()
			}))
		);
	}

	#[tokio::test]
	async fn given_non_authorized_user_then_revoke_user_returns_not_authorized() {
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		let result = revoke_user(&mut conn, "ExistingMachine", UserId::new(76543210987654321)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::AlreadyNotAuthorized {
				machine_name: "ExistingMachine".to_string(),
				entity: DiscordEntity::User(UserId::new(76543210987654321))
			})
		);
	}

	#[tokio::test]
	async fn given_authorized_user_then_revoke_user_returns_success_and_removes_user() {
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		// First permit the user
		permit_user(&mut conn, "ExistingMachine", UserId::new(12345678901234567))
			.await
			.unwrap();

		// Now revoke
		let result = revoke_user(&mut conn, "ExistingMachine", UserId::new(12345678901234567)).await;

		assert_eq!(result, Ok(()));

		// Verify the user was actually removed
		let count: i64 = wake_on_lan_machines_authorized_users::table
			.count()
			.get_result(&mut conn)
			.await
			.unwrap();
		assert_eq!(count, 0);
	}

	// --- permit_role tests ---

	#[tokio::test]
	async fn given_nonexistent_machine_then_permit_role_returns_machine_error() {
		let mut conn = setup_test_db().await;

		let result = permit_role(&mut conn, "NonExistentMachine", RoleId::new(98765432109876543)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::Machine(MachineError::DoesNotExist {
				machine_name: "NonExistentMachine".to_string()
			}))
		);
	}

	#[tokio::test]
	async fn given_already_authorized_role_then_permit_role_returns_already_authorized() {
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		// First permit succeeds
		permit_role(&mut conn, "ExistingMachine", RoleId::new(98765432109876543))
			.await
			.unwrap();

		// Second permit should fail
		let result = permit_role(&mut conn, "ExistingMachine", RoleId::new(98765432109876543)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::AlreadyAuthorized {
				machine_name: "ExistingMachine".to_string(),
				entity: DiscordEntity::Role(RoleId::new(98765432109876543))
			})
		);
	}

	#[tokio::test]
	async fn given_new_role_then_permit_role_returns_success() {
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		let result = permit_role(&mut conn, "ExistingMachine", RoleId::new(98765432109876543)).await;

		assert_eq!(result, Ok(()));
	}

	// --- revoke_role tests ---

	#[tokio::test]
	async fn given_nonexistent_machine_then_revoke_role_returns_machine_error() {
		let mut conn = setup_test_db().await;

		let result = revoke_role(&mut conn, "NonExistentMachine", RoleId::new(98765432109876543)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::Machine(MachineError::DoesNotExist {
				machine_name: "NonExistentMachine".to_string()
			}))
		);
	}

	#[tokio::test]
	async fn given_non_authorized_role_then_revoke_role_returns_not_authorized() {
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		let result = revoke_role(&mut conn, "ExistingMachine", RoleId::new(98765432109876543)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::AlreadyNotAuthorized {
				machine_name: "ExistingMachine".to_string(),
				entity: DiscordEntity::Role(RoleId::new(98765432109876543))
			})
		);
	}

	#[tokio::test]
	async fn given_authorized_role_then_revoke_role_returns_success_and_removes_role() {
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		// First permit the role
		permit_role(&mut conn, "ExistingMachine", RoleId::new(98765432109876543))
			.await
			.unwrap();

		// Now revoke
		let result = revoke_role(&mut conn, "ExistingMachine", RoleId::new(98765432109876543)).await;

		assert_eq!(result, Ok(()));
	}
}
