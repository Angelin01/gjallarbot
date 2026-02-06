use super::ServerError;
use crate::controllers::DiscordEntity;
use crate::db::DbConnection;
use crate::errors::UnexpectedError;
use crate::models::servitor::NewServitorServerAuthorizedUser;
use crate::schema::{servitor_server_authorized_users, servitor_servers};
use diesel::result::DatabaseErrorKind;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use log::info;
use serenity::all::{RoleId, UserId};
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

	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
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

	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

async fn get_server_id<D: DbConnection>(
	conn: &mut D,
	server_name: &str,
) -> Result<i32, ServerError> {
	servitor_servers::table
		.filter(servitor_servers::name.eq(server_name))
		.select(servitor_servers::id)
		.first::<i32>(conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::NotFound => ServerError::DoesNotExist {
				server_name: server_name.into(),
			},
			other => panic!("Unexpected error looking up server: {other}"),
		})
}

pub async fn permit_user<D: DbConnection>(
	conn: &mut D,
	server_name: &str,
	user_id: UserId,
) -> Result<(), AddPermissionError> {
	let server_id = get_server_id(conn, server_name).await?;

	diesel::insert_into(servitor_server_authorized_users::table)
		.values(&NewServitorServerAuthorizedUser {
			server_id,
			user_id: user_id.get() as i64,
		})
		.execute(conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
				AddPermissionError::AlreadyAuthorized {
					server_name: server_name.into(),
					entity: DiscordEntity::User(user_id),
				}
			}
			other => AddPermissionError::Unexpected(UnexpectedError(
				anyhow::Error::new(other)
					.context("Failed to insert authorized user into database"),
			)),
		})?;

	info!("Permitted user {user_id} to operate server {server_name}");
	Ok(())
}

pub async fn revoke_user<D: DbConnection>(
	conn: &mut D,
	server_name: &str,
	user_id: UserId,
) -> Result<(), RemovePermissionError> {
	let server_id = get_server_id(conn, server_name).await.map_err(|e| {
		RemovePermissionError::Server(e)
	})?;

	let affected = diesel::delete(
		servitor_server_authorized_users::table
			.filter(servitor_server_authorized_users::server_id.eq(server_id))
			.filter(servitor_server_authorized_users::user_id.eq(user_id.get() as i64)),
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
			server_name: server_name.into(),
			entity: DiscordEntity::User(user_id),
		});
	}

	info!("Revoked user's {user_id} permission to operate server {server_name}");
	Ok(())
}

pub async fn permit_role<D: DbConnection>(
	conn: &mut D,
	server_name: &str,
	role_id: RoleId,
) -> Result<(), AddPermissionError> {
	use crate::models::servitor::NewServitorServerAuthorizedRole;
	use crate::schema::servitor_server_authorized_roles;

	let server_id = get_server_id(conn, server_name).await?;

	diesel::insert_into(servitor_server_authorized_roles::table)
		.values(&NewServitorServerAuthorizedRole {
			server_id,
			role_id: role_id.get() as i64,
		})
		.execute(conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
				AddPermissionError::AlreadyAuthorized {
					server_name: server_name.into(),
					entity: DiscordEntity::Role(role_id),
				}
			}
			other => AddPermissionError::Unexpected(UnexpectedError(
				anyhow::Error::new(other)
					.context("Failed to insert authorized role into database"),
			)),
		})?;

	info!("Permitted role {role_id} to operate server {server_name}");
	Ok(())
}

pub async fn revoke_role<D: DbConnection>(
	conn: &mut D,
	server_name: &str,
	role_id: RoleId,
) -> Result<(), RemovePermissionError> {
	use crate::schema::servitor_server_authorized_roles;

	let server_id = get_server_id(conn, server_name).await.map_err(|e| {
		RemovePermissionError::Server(e)
	})?;

	let affected = diesel::delete(
		servitor_server_authorized_roles::table
			.filter(servitor_server_authorized_roles::server_id.eq(server_id))
			.filter(servitor_server_authorized_roles::role_id.eq(role_id.get() as i64)),
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
			server_name: server_name.into(),
			entity: DiscordEntity::Role(role_id),
		});
	}

	info!("Revoked role {role_id}'s permission to operate server {server_name}");
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::db::tests::setup_test_db;
	use crate::models::servitor::NewServitorServer;

	async fn insert_test_server<D: DbConnection>(conn: &mut D, name: &str, servitor: &str, unit_name: &str) {
		diesel::insert_into(servitor_servers::table)
			.values(&NewServitorServer {
				name,
				servitor,
				unit_name,
			})
			.execute(conn)
			.await
			.expect("Failed to insert test server");
	}

	// --- permit_user tests ---

	#[tokio::test]
	async fn given_nonexistent_server_then_permit_user_returns_server_error() {
		let mut conn = setup_test_db().await;

		let result = permit_user(&mut conn, "NonExistentServer", UserId::new(12345678901234567)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::Server(ServerError::DoesNotExist {
				server_name: "NonExistentServer".to_string()
			}))
		);
	}

	#[tokio::test]
	async fn given_already_authorized_user_then_permit_user_returns_already_authorized() {
		let mut conn = setup_test_db().await;
		insert_test_server(&mut conn, "ExistingServer", "foo", "bar").await;

		// First permit succeeds
		permit_user(&mut conn, "ExistingServer", UserId::new(12345678901234567))
			.await
			.unwrap();

		// Second permit should fail
		let result = permit_user(&mut conn, "ExistingServer", UserId::new(12345678901234567)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::AlreadyAuthorized {
				server_name: "ExistingServer".to_string(),
				entity: DiscordEntity::User(UserId::new(12345678901234567)),
			})
		);
	}

	#[tokio::test]
	async fn given_new_user_then_permit_user_returns_success() {
		let mut conn = setup_test_db().await;
		insert_test_server(&mut conn, "ExistingServer", "foo", "bar").await;

		let result = permit_user(&mut conn, "ExistingServer", UserId::new(12345678901234567)).await;

		assert_eq!(result, Ok(()));

		// Verify the user was actually inserted
		let count: i64 = servitor_server_authorized_users::table
			.count()
			.get_result(&mut conn)
			.await
			.unwrap();
		assert_eq!(count, 1);
	}

	// --- revoke_user tests ---

	#[tokio::test]
	async fn given_nonexistent_server_then_revoke_user_returns_server_error() {
		let mut conn = setup_test_db().await;

		let result = revoke_user(&mut conn, "NonExistentServer", UserId::new(12345678901234567)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::Server(ServerError::DoesNotExist {
				server_name: "NonExistentServer".to_string()
			}))
		);
	}

	#[tokio::test]
	async fn given_non_authorized_user_then_revoke_user_returns_not_authorized() {
		let mut conn = setup_test_db().await;
		insert_test_server(&mut conn, "ExistingServer", "foo", "bar").await;

		let result = revoke_user(&mut conn, "ExistingServer", UserId::new(76543210987654321)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::AlreadyNotAuthorized {
				server_name: "ExistingServer".to_string(),
				entity: DiscordEntity::User(UserId::new(76543210987654321))
			})
		);
	}

	#[tokio::test]
	async fn given_authorized_user_then_revoke_user_returns_success_and_removes_user() {
		let mut conn = setup_test_db().await;
		insert_test_server(&mut conn, "ExistingServer", "foo", "bar").await;

		// First permit the user
		permit_user(&mut conn, "ExistingServer", UserId::new(12345678901234567))
			.await
			.unwrap();

		// Now revoke
		let result = revoke_user(&mut conn, "ExistingServer", UserId::new(12345678901234567)).await;

		assert_eq!(result, Ok(()));

		// Verify the user was actually removed
		let count: i64 = servitor_server_authorized_users::table
			.count()
			.get_result(&mut conn)
			.await
			.unwrap();
		assert_eq!(count, 0);
	}

	// --- permit_role tests ---

	#[tokio::test]
	async fn given_nonexistent_server_then_permit_role_returns_server_error() {
		let mut conn = setup_test_db().await;

		let result = permit_role(&mut conn, "NonExistentServer", RoleId::new(98765432109876543)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::Server(ServerError::DoesNotExist {
				server_name: "NonExistentServer".to_string()
			}))
		);
	}

	#[tokio::test]
	async fn given_already_authorized_role_then_permit_role_returns_already_authorized() {
		let mut conn = setup_test_db().await;
		insert_test_server(&mut conn, "ExistingServer", "foo", "bar").await;

		// First permit succeeds
		permit_role(&mut conn, "ExistingServer", RoleId::new(98765432109876543))
			.await
			.unwrap();

		// Second permit should fail
		let result = permit_role(&mut conn, "ExistingServer", RoleId::new(98765432109876543)).await;

		assert_eq!(
			result,
			Err(AddPermissionError::AlreadyAuthorized {
				server_name: "ExistingServer".to_string(),
				entity: DiscordEntity::Role(RoleId::new(98765432109876543))
			})
		);
	}

	#[tokio::test]
	async fn given_new_role_then_permit_role_returns_success() {
		let mut conn = setup_test_db().await;
		insert_test_server(&mut conn, "ExistingServer", "foo", "bar").await;

		let result = permit_role(&mut conn, "ExistingServer", RoleId::new(98765432109876543)).await;

		assert_eq!(result, Ok(()));
	}

	// --- revoke_role tests ---

	#[tokio::test]
	async fn given_nonexistent_server_then_revoke_role_returns_server_error() {
		let mut conn = setup_test_db().await;

		let result = revoke_role(&mut conn, "NonExistentServer", RoleId::new(98765432109876543)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::Server(ServerError::DoesNotExist {
				server_name: "NonExistentServer".to_string()
			}))
		);
	}

	#[tokio::test]
	async fn given_non_authorized_role_then_revoke_role_returns_not_authorized() {
		let mut conn = setup_test_db().await;
		insert_test_server(&mut conn, "ExistingServer", "foo", "bar").await;

		let result = revoke_role(&mut conn, "ExistingServer", RoleId::new(98765432109876543)).await;

		assert_eq!(
			result,
			Err(RemovePermissionError::AlreadyNotAuthorized {
				server_name: "ExistingServer".to_string(),
				entity: DiscordEntity::Role(RoleId::new(98765432109876543))
			})
		);
	}

	#[tokio::test]
	async fn given_authorized_role_then_revoke_role_returns_success_and_removes_role() {
		let mut conn = setup_test_db().await;
		insert_test_server(&mut conn, "ExistingServer", "foo", "bar").await;

		// First permit the role
		permit_role(&mut conn, "ExistingServer", RoleId::new(98765432109876543))
			.await
			.unwrap();

		// Now revoke
		let result = revoke_role(&mut conn, "ExistingServer", RoleId::new(98765432109876543)).await;

		assert_eq!(result, Ok(()));
	}
}
