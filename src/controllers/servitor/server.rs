use super::{get_server_info, ServerError};
use crate::controllers::servitor::server::AddServerError::InvalidServitor;
use crate::data::servitor::{ServerInfo, ServitorData};
use crate::data::BotData;
use crate::db::DbConnection;
use crate::errors::UnexpectedError;
use crate::models::servitor::NewServitorServer;
use crate::schema::servitor_servers;
use diesel::result::DatabaseErrorKind;
use diesel_async::RunQueryDsl;
use log::info;
use std::ops::AsyncFnOnce;
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

pub async fn remove_server(data: &BotData, name: &str) -> Result<(), RemoveServerError> {
	if !data.read().await.servitor.contains_key(name) {
		return Err(ServerError::DoesNotExist {
			server_name: name.to_string(),
		})?;
	}

	{
		let mut lock = data.write().await;
		let mut data_write = lock.write();
		data_write.servitor.remove(name);
	}

	info!("Removed servitor server {name}");

	Ok(())
}

pub trait ListServersCallback<T> = AsyncFnOnce(&ServitorData) -> T;
pub async fn list_servers<T, F: ListServersCallback<T>>(data: &BotData, func: F) -> T {
	let read = data.read().await;

	func.async_call_once((&read.servitor,)).await
}

pub trait DescribeServerCallback<T> = AsyncFnOnce(Result<&ServerInfo, ServerError>, &str) -> T;
pub async fn describe_server<T, F: DescribeServerCallback<T>>(
	data: &BotData,
	name: &str,
	func: F,
) -> T {
	let read = data.read().await;

	let server = get_server_info(&read, name).await;

	func.async_call_once((server, name)).await
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::data::servitor::ServerInfo;
	use crate::data::tests::mock_data;
	use crate::db::tests::setup_test_db;
	use crate::models::servitor::ServitorServer;
	use serde_json::json;
	use std::collections::BTreeMap;

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
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar"
				}
			}
		})));

		let result = remove_server(&data, "NonExistingServer").await;

		let expected_data = BTreeMap::from([(
			"SomeServer".to_string(),
			ServerInfo {
				servitor: "foo".to_string(),
				unit_name: "bar".to_string(),
				authorized_users: Default::default(),
				authorized_roles: Default::default(),
			},
		)]);

		assert_eq!(
			result,
			Err(RemoveServerError::Server(ServerError::DoesNotExist {
				server_name: "NonExistingServer".to_string()
			}))
		);
		assert_eq!(data.read().await.servitor, expected_data);
	}

	#[tokio::test]
	async fn given_valid_input_then_remove_server_returns_success_and_removes_server() {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar"
				}
			}
		})));

		let result = remove_server(&data, "SomeServer").await;

		assert_eq!(result, Ok(()));
		assert_eq!(data.read().await.servitor, BTreeMap::new());
	}

	#[tokio::test]
	async fn given_servitor_data_then_list_servers_provides_correct_data_to_callback() {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar"
				}
			}
		})));

		list_servers(&data, async |data| {
			assert_eq!(
				*data,
				BTreeMap::from([(
					"SomeServer".to_string(),
					ServerInfo {
						servitor: "foo".to_string(),
						unit_name: "bar".to_string(),
						authorized_users: Default::default(),
						authorized_roles: Default::default(),
					}
				)])
			)
		})
		.await;
	}

	#[tokio::test]
	async fn given_nonexistent_server_then_describe_server_callbacks_with_error() {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar"
				}
			}
		})));

		describe_server(&data, "NonExistingServer", async |result, name| {
			assert_eq!(name, "NonExistingServer");
			assert_eq!(
				result,
				Err(ServerError::DoesNotExist {
					server_name: "NonExistingServer".to_string()
				})
			)
		})
		.await;
	}

	#[tokio::test]
	async fn given_existing_server_then_describe_server_calls_function_with_server_info() {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar"
				}
			}
		})));

		describe_server(&data, "SomeServer", async |result, name| {
			assert_eq!(name, "SomeServer");
			match result {
				Ok(server) => assert_eq!(
					server,
					&ServerInfo {
						servitor: "foo".to_string(),
						unit_name: "bar".to_string(),
						authorized_users: Default::default(),
						authorized_roles: Default::default(),
					}
				),
				Err(_) => assert!(false, "received error when it was not expected"),
			}
		})
		.await;
	}
}
