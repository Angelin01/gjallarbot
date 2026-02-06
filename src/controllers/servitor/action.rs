use crate::db::DbConnection;
use crate::errors::UnexpectedError;
use crate::models::servitor::{ServitorServer, ServitorServerAuthorizedRole, ServitorServerAuthorizedUser};
use crate::schema::{servitor_server_authorized_roles, servitor_server_authorized_users, servitor_servers};
use crate::services::servitor::{ServitorController, ServitorError, UnitStatus};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use log::info;
use serenity::all::{Member, User, UserId};
use std::collections::BTreeMap;
use std::ops::AsyncFnOnce;
use thiserror::Error;

use super::ServerError;

#[derive(Debug, Error, PartialEq)]
pub enum ExecuteServitorActionError {
	#[error(transparent)]
	Server(#[from] ServerError),

	#[error("the configured Servitor instance {servitor_name} for server {server_name} no longer exists")]
	InvalidServitor {
		server_name: String,
		servitor_name: String,
	},

	#[error("User {user} is not authorized to operate Servitor server {server_name}")]
	Unauthorized { user: UserId, server_name: String },

	#[error(transparent)]
	Servitor(#[from] ServitorError),

	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

pub async fn start<D: DbConnection, S: ServitorController>(
	conn: &mut D,
	servitor_handlers: &BTreeMap<String, S>,
	server_name: &str,
	author: &User,
	member: Option<&Member>,
) -> Result<(), ExecuteServitorActionError> {
	execute_action(
		conn,
		servitor_handlers,
		server_name,
		author,
		member,
		async |h: &S, u: &str| {
			info!("Running start for Servitor server {server_name}");
			h.start(u).await
		},
	)
	.await
}

pub async fn stop<D: DbConnection, S: ServitorController>(
	conn: &mut D,
	servitor_handlers: &BTreeMap<String, S>,
	server_name: &str,
	author: &User,
	member: Option<&Member>,
) -> Result<(), ExecuteServitorActionError> {
	execute_action(
		conn,
		servitor_handlers,
		server_name,
		author,
		member,
		async |h: &S, u: &str| {
			info!("Running stop for Servitor server {server_name}");
			h.stop(u).await
		},
	)
	.await
}

pub async fn restart<D: DbConnection, S: ServitorController>(
	conn: &mut D,
	servitor_handlers: &BTreeMap<String, S>,
	server_name: &str,
	author: &User,
	member: Option<&Member>,
) -> Result<(), ExecuteServitorActionError> {
	execute_action(
		conn,
		servitor_handlers,
		server_name,
		author,
		member,
		async |h: &S, u: &str| {
			info!("Running restart for Servitor server {server_name}");
			h.restart(u).await
		},
	)
	.await
}

pub async fn reload<D: DbConnection, S: ServitorController>(
	conn: &mut D,
	servitor_handlers: &BTreeMap<String, S>,
	server_name: &str,
	author: &User,
	member: Option<&Member>,
) -> Result<(), ExecuteServitorActionError> {
	execute_action(
		conn,
		servitor_handlers,
		server_name,
		author,
		member,
		async |h: &S, u: &str| {
			info!("Running reload for Servitor server {server_name}");
			h.reload(u).await
		},
	)
	.await
}

pub async fn status<D: DbConnection, S: ServitorController>(
	conn: &mut D,
	servitor_handlers: &BTreeMap<String, S>,
	server_name: &str,
	author: &User,
	member: Option<&Member>,
) -> Result<UnitStatus, ExecuteServitorActionError> {
	execute_action(
		conn,
		servitor_handlers,
		server_name,
		author,
		member,
		async |h: &S, u: &str| {
			info!("Running status for Servitor server {server_name}");
			h.status(u).await
		},
	)
	.await
}

async fn execute_action<D, S, F, T>(
	conn: &mut D,
	servitor_handlers: &BTreeMap<String, S>,
	server_name: &str,
	author: &User,
	member: Option<&Member>,
	action: F,
) -> Result<T, ExecuteServitorActionError>
where
	D: DbConnection,
	S: ServitorController,
	F: AsyncFnOnce(&S, &str) -> Result<T, ServitorError>,
{
	let server: ServitorServer = servitor_servers::table
		.filter(servitor_servers::name.eq(server_name))
		.first(conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::NotFound => ServerError::DoesNotExist {
				server_name: server_name.into(),
			}
			.into(),
			other => ExecuteServitorActionError::Unexpected(UnexpectedError(
				anyhow::Error::new(other).context("Failed to query server from database"),
			)),
		})?;

	let authorized_users: Vec<ServitorServerAuthorizedUser> =
		servitor_server_authorized_users::table
			.filter(servitor_server_authorized_users::server_id.eq(server.id))
			.load(conn)
			.await
			.map_err(|e| {
				ExecuteServitorActionError::Unexpected(UnexpectedError(
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
				ExecuteServitorActionError::Unexpected(UnexpectedError(
					anyhow::Error::new(e)
						.context("Failed to query authorized roles from database"),
				))
			})?;

	let user_authorized = authorized_users
		.iter()
		.any(|u| UserId::new(u.user_id as u64) == author.id);

	let role_authorized = member.is_some_and(|m| {
		m.roles.iter().any(|role| {
			authorized_roles
				.iter()
				.any(|r| serenity::all::RoleId::new(r.role_id as u64) == *role)
		})
	});

	if !user_authorized && !role_authorized {
		return Err(ExecuteServitorActionError::Unauthorized {
			user: author.id,
			server_name: server_name.to_string(),
		});
	}

	let servitor_handler = servitor_handlers.get(&server.servitor).ok_or(
		ExecuteServitorActionError::InvalidServitor {
			server_name: server_name.to_string(),
			servitor_name: server.servitor.to_string(),
		},
	)?;

	Ok(action(servitor_handler, &server.unit_name).await?)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::controllers::tests::{mock_author_dms, mock_author_guild};
	use crate::db::tests::setup_test_db;
	use crate::models::servitor::{NewServitorServer, NewServitorServerAuthorizedRole, NewServitorServerAuthorizedUser};
	use crate::services::servitor::tests::MockServitorController;
	use diesel::SqliteConnection;
	use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
	use rstest::rstest;
	use serenity::all::RoleId;
	use std::fmt::Debug;

	type TestConn = SyncConnectionWrapper<SqliteConnection>;

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

	async fn get_server_id<D: DbConnection>(conn: &mut D, name: &str) -> i32 {
		servitor_servers::table
			.filter(servitor_servers::name.eq(name))
			.select(servitor_servers::id)
			.first::<i32>(conn)
			.await
			.expect("Failed to get server id")
	}

	async fn permit_test_user<D: DbConnection>(conn: &mut D, server_name: &str, user_id: u64) {
		let server_id = get_server_id(conn, server_name).await;
		diesel::insert_into(servitor_server_authorized_users::table)
			.values(&NewServitorServerAuthorizedUser {
				server_id,
				user_id: user_id as i64,
			})
			.execute(conn)
			.await
			.expect("Failed to insert authorized user");
	}

	async fn permit_test_role<D: DbConnection>(conn: &mut D, server_name: &str, role_id: u64) {
		let server_id = get_server_id(conn, server_name).await;
		diesel::insert_into(servitor_server_authorized_roles::table)
			.values(&NewServitorServerAuthorizedRole {
				server_id,
				role_id: role_id as i64,
			})
			.execute(conn)
			.await
			.expect("Failed to insert authorized role");
	}

	fn mock_servitor_handlers() -> BTreeMap<String, MockServitorController> {
		let mut handlers = BTreeMap::new();
		handlers.insert("foo".to_string(), MockServitorController::new());
		handlers
	}

	#[rstest]
	#[case(start)]
	#[case(stop)]
	#[case(restart)]
	#[case(reload)]
	#[case(status)]
	#[tokio::test]
	async fn given_invalid_server_name_then_action_returns_invalid_server_error<
		T: Debug + PartialEq,
	>(
		#[case] action: impl AsyncFnOnce(
			&mut TestConn,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
	) {
		let mut conn: TestConn = setup_test_db().await;
		let serv = mock_servitor_handlers();
		let (author, member) = mock_author_dms(UserId::new(12345678901234567));

		let result = action(&mut conn, &serv, "NonExistingServer", &author, member.as_ref()).await;

		assert_eq!(
			result,
			Err(ExecuteServitorActionError::Server(
				ServerError::DoesNotExist {
					server_name: "NonExistingServer".to_string(),
				}
			))
		);
		serv.values().for_each(|s| s.assert_not_called());
	}

	#[rstest]
	#[case(start)]
	#[case(stop)]
	#[case(restart)]
	#[case(reload)]
	#[case(status)]
	#[tokio::test]
	async fn given_server_with_invalid_servitor_configured_then_action_returns_invalid_servitor_error<
		T: Debug + PartialEq,
	>(
		#[case] action: impl AsyncFnOnce(
			&mut TestConn,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
	) {
		let mut conn: TestConn = setup_test_db().await;
		insert_test_server(&mut conn, "SomeServer", "foo", "bar").await;
		permit_test_user(&mut conn, "SomeServer", 12345678901234567).await;

		let mut serv = mock_servitor_handlers();
		serv.remove("foo");
		let (author, member) = mock_author_dms(UserId::new(12345678901234567u64));

		let result = action(&mut conn, &serv, "SomeServer", &author, member.as_ref()).await;

		assert_eq!(
			result,
			Err(ExecuteServitorActionError::InvalidServitor {
				server_name: "SomeServer".to_string(),
				servitor_name: "foo".to_string(),
			})
		);
		serv.values().for_each(|s| s.assert_not_called());
	}

	#[rstest]
	#[case(start)]
	#[case(stop)]
	#[case(restart)]
	#[case(reload)]
	#[case(status)]
	#[tokio::test]
	async fn given_dm_call_but_user_not_in_allowed_list_then_action_returns_unauthorized_error<
		T: Debug + PartialEq,
	>(
		#[case] action: impl AsyncFnOnce(
			&mut TestConn,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
	) {
		let mut conn: TestConn = setup_test_db().await;
		insert_test_server(&mut conn, "SomeServer", "foo", "bar").await;
		permit_test_user(&mut conn, "SomeServer", 98765432109876543).await;
		permit_test_user(&mut conn, "SomeServer", 98765432109876542).await;
		permit_test_role(&mut conn, "SomeServer", 98765432109876541).await;
		permit_test_role(&mut conn, "SomeServer", 98765432109876540).await;

		let serv = mock_servitor_handlers();
		let (author, member) = mock_author_dms(UserId::new(12345678901234567));

		let result = action(&mut conn, &serv, "SomeServer", &author, member.as_ref()).await;

		assert_eq!(
			result,
			Err(ExecuteServitorActionError::Unauthorized {
				user: UserId::new(12345678901234567),
				server_name: "SomeServer".to_string(),
			})
		);
		serv.values().for_each(|s| s.assert_not_called());
	}

	#[rstest]
	#[case(start)]
	#[case(stop)]
	#[case(restart)]
	#[case(reload)]
	#[case(status)]
	#[tokio::test]
	async fn given_guild_call_but_user_not_in_allowed_list_then_action_returns_unauthorized_error<
		T: Debug + PartialEq,
	>(
		#[case] action: impl AsyncFnOnce(
			&mut TestConn,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
	) {
		let mut conn: TestConn = setup_test_db().await;
		insert_test_server(&mut conn, "SomeServer", "foo", "bar").await;
		permit_test_user(&mut conn, "SomeServer", 98765432109876543).await;
		permit_test_user(&mut conn, "SomeServer", 98765432109876542).await;
		permit_test_role(&mut conn, "SomeServer", 98765432109876541).await;
		permit_test_role(&mut conn, "SomeServer", 98765432109876540).await;

		let serv = mock_servitor_handlers();
		let (author, member) = mock_author_guild(UserId::new(12345678901234567), vec![]);

		let result = action(&mut conn, &serv, "SomeServer", &author, member.as_ref()).await;

		assert_eq!(
			result,
			Err(ExecuteServitorActionError::Unauthorized {
				user: UserId::new(12345678901234567),
				server_name: "SomeServer".to_string(),
			})
		);
		serv.values().for_each(|s| s.assert_not_called());
	}

	#[rstest]
	#[case(start, (1, 0, 0, 0, 0))]
	#[case(stop, (0, 1, 0, 0, 0))]
	#[case(restart, (0, 0, 1, 0, 0))]
	#[case(reload, (0, 0, 0, 1, 0))]
	#[case(status, (0, 0, 0, 0, 1))]
	#[tokio::test]
	async fn given_unexpected_servitor_error_then_action_should_return_servitor_error<
		T: Debug + PartialEq,
	>(
		#[case] action: impl AsyncFnOnce(
			&mut TestConn,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
		#[case] calls: (usize, usize, usize, usize, usize),
	) {
		let mut conn: TestConn = setup_test_db().await;
		insert_test_server(&mut conn, "SomeServer", "foo", "bar").await;
		permit_test_user(&mut conn, "SomeServer", 12345678901234567).await;

		let serv = mock_servitor_handlers();
		serv["foo"].set_error(ServitorError::Unauthorized).await;
		let (author, member) = mock_author_dms(UserId::new(12345678901234567u64));

		let result = action(&mut conn, &serv, "SomeServer", &author, member.as_ref()).await;

		assert_eq!(
			result,
			Err(ExecuteServitorActionError::Servitor(
				ServitorError::Unauthorized
			))
		);
		serv["foo"].assert_called_times(calls.0, calls.1, calls.2, calls.3, calls.4);
	}

	#[rstest]
	#[case(start, (1, 0, 0, 0, 0), ())]
	#[case(stop, (0, 1, 0, 0, 0), ())]
	#[case(restart, (0, 0, 1, 0, 0), ())]
	#[case(reload, (0, 0, 0, 1, 0), ())]
	#[case(status, (0, 0, 0, 0, 1), MockServitorController::default_status("bar"))]
	#[tokio::test]
	async fn given_dm_call_and_user_in_allowed_list_then_should_action_server<
		T: Debug + PartialEq,
	>(
		#[case] action: impl AsyncFnOnce(
			&mut TestConn,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
		#[case] calls: (usize, usize, usize, usize, usize),
		#[case] expected_result: T,
	) {
		let mut conn: TestConn = setup_test_db().await;
		insert_test_server(&mut conn, "SomeServer", "foo", "bar").await;
		permit_test_user(&mut conn, "SomeServer", 12345678901234567).await;

		let serv = mock_servitor_handlers();
		let (author, member) = mock_author_dms(UserId::new(12345678901234567u64));

		let result = action(&mut conn, &serv, "SomeServer", &author, member.as_ref()).await;

		assert_eq!(result, Ok(expected_result));
		serv["foo"].assert_called_times(calls.0, calls.1, calls.2, calls.3, calls.4);
	}

	#[rstest]
	#[case(start, (1, 0, 0, 0, 0), ())]
	#[case(stop, (0, 1, 0, 0, 0), ())]
	#[case(restart, (0, 0, 1, 0, 0), ())]
	#[case(reload, (0, 0, 0, 1, 0), ())]
	#[case(status, (0, 0, 0, 0, 1), MockServitorController::default_status("bar"))]
	#[tokio::test]
	async fn given_guild_call_and_user_in_allowed_list_then_should_action_server<
		T: Debug + PartialEq,
	>(
		#[case] action: impl AsyncFnOnce(
			&mut TestConn,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
		#[case] calls: (usize, usize, usize, usize, usize),
		#[case] expected_result: T,
	) {
		let mut conn: TestConn = setup_test_db().await;
		insert_test_server(&mut conn, "SomeServer", "foo", "bar").await;
		permit_test_user(&mut conn, "SomeServer", 12345678901234567).await;

		let serv = mock_servitor_handlers();
		let (author, member) = mock_author_guild(UserId::new(12345678901234567u64), vec![]);

		let result = action(&mut conn, &serv, "SomeServer", &author, member.as_ref()).await;

		assert_eq!(result, Ok(expected_result));
		serv["foo"].assert_called_times(calls.0, calls.1, calls.2, calls.3, calls.4);
	}

	#[rstest]
	#[case(start, (1, 0, 0, 0, 0), ())]
	#[case(stop, (0, 1, 0, 0, 0), ())]
	#[case(restart, (0, 0, 1, 0, 0), ())]
	#[case(reload, (0, 0, 0, 1, 0), ())]
	#[case(status, (0, 0, 0, 0, 1), MockServitorController::default_status("bar"))]
	#[tokio::test]
	async fn given_guild_call_and_users_role_in_allowed_list_then_should_action_server<
		T: Debug + PartialEq,
	>(
		#[case] action: impl AsyncFnOnce(
			&mut TestConn,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
		#[case] calls: (usize, usize, usize, usize, usize),
		#[case] expected_result: T,
	) {
		let mut conn: TestConn = setup_test_db().await;
		insert_test_server(&mut conn, "SomeServer", "foo", "bar").await;
		permit_test_role(&mut conn, "SomeServer", 98765432109876543).await;

		let serv = mock_servitor_handlers();
		let (author, member) = mock_author_guild(
			UserId::new(12345678901234567u64),
			vec![RoleId::new(98765432109876543u64)],
		);

		let result = action(&mut conn, &serv, "SomeServer", &author, member.as_ref()).await;

		assert_eq!(result, Ok(expected_result));
		serv["foo"].assert_called_times(calls.0, calls.1, calls.2, calls.3, calls.4);
	}
}
