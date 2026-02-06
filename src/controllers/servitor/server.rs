use super::ServerError;
use crate::controllers::servitor::server::AddServerError::InvalidServitor;
use crate::db::DbConnection;
use crate::errors::UnexpectedError;
use crate::models::servitor::{
	NewServitorServer, ServitorServer, ServitorServerAuthorizedRole, ServitorServerAuthorizedUser,
};
use crate::schema::{servitor_server_authorized_roles, servitor_server_authorized_users, servitor_servers};
use diesel::result::DatabaseErrorKind;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use log::info;
use poise::serenity_prelude::{RoleId, UserId};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum AddServerError {
	#[error("no such servitor instance {name} configured")]
	InvalidServitor { name: String },

	#[error(transparent)]
	Server(#[from] ServerError),

	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

#[derive(Debug, Error, PartialEq)]
pub enum RemoveServerError {
	#[error(transparent)]
	Server(#[from] ServerError),

	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

#[derive(Debug, Error, PartialEq)]
pub enum ListServersError {
	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

#[derive(Debug, Error, PartialEq)]
pub enum DescribeServerError {
	#[error(transparent)]
	Server(#[from] ServerError),

	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

#[derive(Debug)]
pub struct ServerDescription {
	pub server: ServitorServer,
	pub authorized_users: Vec<UserId>,
	pub authorized_roles: Vec<RoleId>,
}

pub async fn add_server<D: DbConnection>(
	conn: &mut D,
	servitor_names: &[String],
	name: &str,
	servitor: &str,
	unit_name: &str,
) -> Result<(), AddServerError> {
	if !servitor_names.iter().any(|s| s == servitor) {
		return Err(InvalidServitor {
			name: servitor.to_string(),
		});
	}

	let new_server = NewServitorServer {
		name,
		servitor,
		unit_name,
	};

	diesel::insert_into(servitor_servers::table)
		.values(&new_server)
		.execute(conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
				ServerError::AlreadyExists {
					server_name: name.into(),
				}
				.into()
			}
			other => AddServerError::Unexpected(UnexpectedError(
				anyhow::Error::new(other).context("Failed to insert server into database"),
			)),
		})?;

	info!("Added servitor server {name} with Servitor {servitor} and unit_name {unit_name}");

	Ok(())
}

pub async fn remove_server<D: DbConnection>(
	conn: &mut D,
	name: &str,
) -> Result<(), RemoveServerError> {
	let affected = diesel::delete(
		servitor_servers::table.filter(servitor_servers::name.eq(name)),
	)
	.execute(conn)
	.await
	.map_err(|e| {
		RemoveServerError::Unexpected(UnexpectedError(
			anyhow::Error::new(e).context("Failed to delete server from database"),
		))
	})?;

	if affected == 0 {
		return Err(ServerError::DoesNotExist {
			server_name: name.into(),
		}
		.into());
	}

	info!("Removed servitor server {name}");

	Ok(())
}

pub async fn list_servers<D: DbConnection>(
	conn: &mut D,
) -> Result<Vec<ServitorServer>, ListServersError> {
	servitor_servers::table
		.load(conn)
		.await
		.map_err(|e| {
			ListServersError::Unexpected(UnexpectedError(
				anyhow::Error::new(e).context("Failed to load servers from database"),
			))
		})
}

pub async fn describe_server<D: DbConnection>(
	conn: &mut D,
	name: &str,
) -> Result<ServerDescription, DescribeServerError> {
	let server: ServitorServer = servitor_servers::table
		.filter(servitor_servers::name.eq(name))
		.first(conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::NotFound => ServerError::DoesNotExist {
				server_name: name.into(),
			}
			.into(),
			other => DescribeServerError::Unexpected(UnexpectedError(
				anyhow::Error::new(other).context("Failed to query server from database"),
			)),
		})?;

	let authorized_users: Vec<ServitorServerAuthorizedUser> =
		servitor_server_authorized_users::table
			.filter(servitor_server_authorized_users::server_id.eq(server.id))
			.load(conn)
			.await
			.map_err(|e| {
				DescribeServerError::Unexpected(UnexpectedError(
					anyhow::Error::new(e)
						.context("Failed to query authorized users from database"),
				))
			})?;

	let authorized_roles: Vec<ServitorServerAuthorizedRole> =
		servitor_server_authorized_roles::table
			.filter(servitor_server_authorized_roles::server_id.eq(server.id))
			.load(conn)
			.await
			.map_err(|e| {
				DescribeServerError::Unexpected(UnexpectedError(
					anyhow::Error::new(e)
						.context("Failed to query authorized roles from database"),
				))
			})?;

	Ok(ServerDescription {
		server,
		authorized_users: authorized_users
			.into_iter()
			.map(|u| UserId::new(u.user_id as u64))
			.collect(),
		authorized_roles: authorized_roles
			.into_iter()
			.map(|r| RoleId::new(r.role_id as u64))
			.collect(),
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::db::tests::setup_test_db;
	use crate::models::servitor::{
		NewServitorServerAuthorizedRole, NewServitorServerAuthorizedUser,
	};

	async fn insert_test_server<D: DbConnection>(
		conn: &mut D,
		name: &str,
		servitor: &str,
		unit_name: &str,
	) {
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

	async fn get_all_servers<D: DbConnection>(conn: &mut D) -> Vec<ServitorServer> {
		servitor_servers::table
			.load(conn)
			.await
			.expect("Failed to load servers")
	}

	#[tokio::test]
	async fn given_invalid_servitor_name_then_add_server_returns_invalid_servitor_error_and_does_not_update_data(
	) {
		let mut conn = setup_test_db().await;
		let servitor_names = vec!["foo".to_string()];

		let result =
			add_server(&mut conn, &servitor_names, "test", "NonExistingServitor", "some_name")
				.await;

		assert_eq!(
			result,
			Err(AddServerError::InvalidServitor {
				name: "NonExistingServitor".to_string()
			})
		);
		assert!(get_all_servers(&mut conn).await.is_empty());
	}

	#[tokio::test]
	async fn given_duplicate_name_then_add_server_returns_error_and_does_not_update_data() {
		let mut conn = setup_test_db().await;
		let servitor_names = vec!["foo".to_string()];

		insert_test_server(&mut conn, "SomeServer", "foo", "bar").await;

		let result =
			add_server(&mut conn, &servitor_names, "SomeServer", "foo", "some_name").await;

		assert_eq!(
			result,
			Err(AddServerError::Server(ServerError::AlreadyExists {
				server_name: "SomeServer".to_string()
			}))
		);

		let servers = get_all_servers(&mut conn).await;
		assert_eq!(servers.len(), 1);
		assert_eq!(servers[0].name, "SomeServer");
	}

	#[tokio::test]
	async fn given_valid_input_then_add_server_returns_success_and_adds_new_server() {
		let mut conn = setup_test_db().await;
		let servitor_names = vec!["foo".to_string()];

		let result =
			add_server(&mut conn, &servitor_names, "NewServer", "foo", "some_name").await;

		assert_eq!(result, Ok(()));

		let servers = get_all_servers(&mut conn).await;
		assert_eq!(servers.len(), 1);
		assert_eq!(servers[0].name, "NewServer");
		assert_eq!(servers[0].servitor, "foo");
		assert_eq!(servers[0].unit_name, "some_name");
	}

	#[tokio::test]
	async fn given_invalid_server_name_then_remove_server_returns_error_and_does_not_update_data() {
		let mut conn = setup_test_db().await;

		insert_test_server(&mut conn, "SomeServer", "foo", "bar").await;

		let result = remove_server(&mut conn, "NonExistingServer").await;

		assert_eq!(
			result,
			Err(RemoveServerError::Server(ServerError::DoesNotExist {
				server_name: "NonExistingServer".to_string()
			}))
		);
		// Original server should still exist
		let servers = get_all_servers(&mut conn).await;
		assert_eq!(servers.len(), 1);
		assert_eq!(servers[0].name, "SomeServer");
	}

	#[tokio::test]
	async fn given_valid_input_then_remove_server_returns_success_and_removes_server() {
		let mut conn = setup_test_db().await;

		insert_test_server(&mut conn, "SomeServer", "foo", "bar").await;

		let result = remove_server(&mut conn, "SomeServer").await;

		assert_eq!(result, Ok(()));
		assert!(get_all_servers(&mut conn).await.is_empty());
	}

	#[tokio::test]
	async fn given_no_servers_then_list_servers_returns_empty_vec() {
		let mut conn = setup_test_db().await;

		let result = list_servers(&mut conn).await.unwrap();

		assert!(result.is_empty());
	}

	#[tokio::test]
	async fn given_servers_exist_then_list_servers_returns_all_servers() {
		let mut conn = setup_test_db().await;

		insert_test_server(&mut conn, "ServerA", "foo", "unit_a.service").await;
		insert_test_server(&mut conn, "ServerB", "bar", "unit_b.service").await;

		let result = list_servers(&mut conn).await.unwrap();

		assert_eq!(result.len(), 2);
		assert_eq!(result[0].name, "ServerA");
		assert_eq!(result[0].servitor, "foo");
		assert_eq!(result[0].unit_name, "unit_a.service");
		assert_eq!(result[1].name, "ServerB");
		assert_eq!(result[1].servitor, "bar");
		assert_eq!(result[1].unit_name, "unit_b.service");
	}

	#[tokio::test]
	async fn given_nonexistent_server_then_describe_server_returns_error() {
		let mut conn = setup_test_db().await;

		insert_test_server(&mut conn, "ExistingServer", "foo", "bar").await;

		let result = describe_server(&mut conn, "NonExistentServer").await;

		assert_eq!(
			result.unwrap_err(),
			DescribeServerError::Server(ServerError::DoesNotExist {
				server_name: "NonExistentServer".into(),
			})
		);
	}

	#[tokio::test]
	async fn given_existing_server_then_describe_server_returns_server_info() {
		let mut conn = setup_test_db().await;

		insert_test_server(&mut conn, "ExistingServer", "foo", "bar").await;

		let result = describe_server(&mut conn, "ExistingServer").await.unwrap();

		assert_eq!(result.server.name, "ExistingServer");
		assert_eq!(result.server.servitor, "foo");
		assert_eq!(result.server.unit_name, "bar");
		assert!(result.authorized_users.is_empty());
		assert!(result.authorized_roles.is_empty());
	}

	#[tokio::test]
	async fn given_server_with_authorized_users_and_roles_then_describe_server_returns_them() {
		let mut conn = setup_test_db().await;

		insert_test_server(&mut conn, "ExistingServer", "foo", "bar").await;

		// Get the server id
		let server: ServitorServer = servitor_servers::table
			.filter(servitor_servers::name.eq("ExistingServer"))
			.first(&mut conn)
			.await
			.unwrap();

		// Insert authorized users
		diesel::insert_into(servitor_server_authorized_users::table)
			.values(&NewServitorServerAuthorizedUser {
				server_id: server.id,
				user_id: 12345678901234567,
			})
			.execute(&mut conn)
			.await
			.unwrap();

		// Insert authorized roles
		diesel::insert_into(servitor_server_authorized_roles::table)
			.values(&NewServitorServerAuthorizedRole {
				server_id: server.id,
				role_id: 98765432109876543,
			})
			.execute(&mut conn)
			.await
			.unwrap();

		let result = describe_server(&mut conn, "ExistingServer").await.unwrap();

		assert_eq!(result.server.name, "ExistingServer");
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
