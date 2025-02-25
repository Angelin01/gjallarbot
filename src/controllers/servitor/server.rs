use super::ServerError;
use crate::controllers::servitor::server::AddServerError::InvalidServitor;
use crate::data::BotData;
use crate::services::servitor::ServitorController;
use std::collections::BTreeMap;
use log::info;
use thiserror::Error;
use crate::data::servitor::ServerInfo;

#[derive(Debug, Error, PartialEq)]
pub enum AddServerError {
	#[error("no such servitor instance {name} configured")]
	InvalidServitor { name: String },

	#[error(transparent)]
	Server(#[from] ServerError),
}

pub async fn add_server<S: ServitorController>(
	data: &BotData,
	servitor_handlers: &BTreeMap<String, S>,
	name: &str,
	servitor: &str,
	unit_name: &str,
) -> Result<(), AddServerError> {
	if !servitor_handlers.contains_key(servitor) {
		return Err(InvalidServitor {
			name: servitor.to_string(),
		});
	}

	if data.read().await.servitor.contains_key(name) {
		return Err(ServerError::AlreadyExists {
			server_name: name.to_string(),
		})?;
	}

	{
		let mut lock = data.write().await;
		let mut data_write = lock.write();
		data_write.servitor.insert(
			name.to_string(),
			ServerInfo {
				servitor: servitor.to_string(),
				unit_name: unit_name.to_string(),
				authorized_users: Default::default(),
				authorized_roles: Default::default(),
			}
		);
	}

	info!("Added servitor server {name} with Servitor {servitor} and unit_name {unit_name}");

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::data::servitor::ServerInfo;
	use crate::data::tests::mock_data;
	use crate::services::servitor::tests::{controllers_from_bot_data, MockServitorController};
	use serde_json::json;
	use std::collections::BTreeMap;

	#[tokio::test]
	async fn given_invalid_servitor_name_then_add_server_returns_invalid_servitor_error_and_does_not_update_data(
	) {
		let data = mock_data(Some(json!({
			"servitor": {}
		})));
		let serv = controllers_from_bot_data(&data).await;

		let result = add_server(&data, &serv, "test", "NonExistingServitor", "some_name").await;

		assert_eq!(
			result,
			Err(AddServerError::InvalidServitor {
				name: "NonExistingServitor".to_string()
			})
		);
		assert_eq!(data.read().await.servitor, BTreeMap::new());
		serv.values().for_each(MockServitorController::assert_not_called);
	}

	#[tokio::test]
	async fn given_duplicate_name_then_add_server_returns_error_and_does_not_update_data() {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar"
				}
			}
		})));
		let serv = controllers_from_bot_data(&data).await;

		let result = add_server(&data, &serv, "SomeServer", "foo", "some_name").await;

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
			Err(AddServerError::Server(ServerError::AlreadyExists {
				server_name: "SomeServer".to_string()
			}))
		);
		assert_eq!(data.read().await.servitor, expected_data);

		serv.values().for_each(MockServitorController::assert_not_called);
	}

	#[tokio::test]
	async fn given_valid_input_then_add_server_returns_success_and_adds_new_server() {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar"
				}
			}
		})));
		let serv = controllers_from_bot_data(&data).await;

		let result = add_server(&data, &serv, "NewServer", "foo", "some_name").await;

		let expected_data = BTreeMap::from([
			(
				"SomeServer".to_string(),
				ServerInfo {
					servitor: "foo".to_string(),
					unit_name: "bar".to_string(),
					authorized_users: Default::default(),
					authorized_roles: Default::default(),
				},
			),
			(
				"NewServer".to_string(),
				ServerInfo {
					servitor: "foo".to_string(),
					unit_name: "some_name".to_string(),
					authorized_users: Default::default(),
					authorized_roles: Default::default(),
				},
			),
		]);

		assert_eq!(result, Ok(()));
		assert_eq!(data.read().await.servitor, expected_data);

		serv.values().for_each(MockServitorController::assert_not_called);
	}
}
