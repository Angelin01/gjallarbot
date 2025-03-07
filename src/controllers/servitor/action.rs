use super::super::is_user_authorized;
use crate::controllers::servitor::{get_server_info, ServerError};
use crate::data::BotData;
use crate::services::servitor::{ServitorController, ServitorError, UnitStatus};
use log::info;
use serenity::all::{Member, User, UserId};
use std::collections::BTreeMap;
use std::ops::AsyncFnOnce;
use thiserror::Error;

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
}

pub async fn start<S: ServitorController>(
	data: &BotData,
	servitor_handlers: &BTreeMap<String, S>,
	server_name: &str,
	author: &User,
	member: Option<&Member>,
) -> Result<(), ExecuteServitorActionError> {
	execute_action(
		data,
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

pub async fn stop<S: ServitorController>(
	data: &BotData,
	servitor_handlers: &BTreeMap<String, S>,
	server_name: &str,
	author: &User,
	member: Option<&Member>,
) -> Result<(), ExecuteServitorActionError> {
	execute_action(
		data,
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

pub async fn restart<S: ServitorController>(
	data: &BotData,
	servitor_handlers: &BTreeMap<String, S>,
	server_name: &str,
	author: &User,
	member: Option<&Member>,
) -> Result<(), ExecuteServitorActionError> {
	execute_action(
		data,
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

pub async fn reload<S: ServitorController>(
	data: &BotData,
	servitor_handlers: &BTreeMap<String, S>,
	server_name: &str,
	author: &User,
	member: Option<&Member>,
) -> Result<(), ExecuteServitorActionError> {
	execute_action(
		data,
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

pub async fn status<S: ServitorController>(
	data: &BotData,
	servitor_handlers: &BTreeMap<String, S>,
	server_name: &str,
	author: &User,
	member: Option<&Member>,
) -> Result<UnitStatus, ExecuteServitorActionError> {
	execute_action(
		data,
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

async fn execute_action<S, F, T>(
	data: &BotData,
	servitor_handlers: &BTreeMap<String, S>,
	server_name: &str,
	author: &User,
	member: Option<&Member>,
	action: F,
) -> Result<T, ExecuteServitorActionError>
where
	S: ServitorController,
	F: AsyncFnOnce(&S, &str) -> Result<T, ServitorError>,
{
	let read = data.read().await;

	let server_info = get_server_info(&read, server_name).await?;

	if !is_user_authorized(author, member, server_info) {
		return Err(ExecuteServitorActionError::Unauthorized {
			user: author.id,
			server_name: server_name.to_string(),
		});
	}

	let servitor_handler = servitor_handlers.get(&server_info.servitor).ok_or(
		ExecuteServitorActionError::InvalidServitor {
			server_name: server_name.to_string(),
			servitor_name: server_info.servitor.to_string(),
		},
	)?;

	Ok(action(servitor_handler, &server_info.unit_name.clone()).await?)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::controllers::tests::{mock_author_dms, mock_author_guild};
	use crate::data::tests::mock_data;
	use crate::services::servitor::tests::{controllers_from_bot_data, MockServitorController};
	use rstest::rstest;
	use serde_json::json;
	use serenity::all::RoleId;
	use std::fmt::Debug;

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
			&BotData,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
	) {
		let data = mock_data(Some(json!({
			"servitor": {}
		})));
		let serv = controllers_from_bot_data(&data).await;
		let (author, member) = mock_author_dms(UserId::new(12345678901234567));

		let result = action(&data, &serv, "NonExistingServer", &author, member.as_ref()).await;

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
			&BotData,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
	) {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar",
					"authorized_users": [12345678901234567u64],
				}
			}
		})));

		let mut serv = controllers_from_bot_data(&data).await;
		serv.remove("foo");
		let (author, member) = mock_author_dms(UserId::new(12345678901234567u64));

		let result = action(&data, &serv, "SomeServer", &author, member.as_ref()).await;

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
			&BotData,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
	) {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar",
					"authorized_users": [98765432109876543i64, 98765432109876542i64],
					"authorized_roles": [98765432109876541i64, 98765432109876540i64]
				}
			}
		})));
		let serv = controllers_from_bot_data(&data).await;
		let (author, member) = mock_author_dms(UserId::new(12345678901234567));

		let result = action(&data, &serv, "SomeServer", &author, member.as_ref()).await;

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
			&BotData,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
	) {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar",
					"authorized_users": [98765432109876543u64, 98765432109876542u64],
					"authorized_roles": [98765432109876541u64, 98765432109876540u64]
				}
			}
		})));
		let serv = controllers_from_bot_data(&data).await;
		let (author, member) = mock_author_guild(UserId::new(12345678901234567), vec![]);

		let result = action(&data, &serv, "SomeServer", &author, member.as_ref()).await;

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
			&BotData,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
		#[case] calls: (usize, usize, usize, usize, usize),
	) {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar",
					"authorized_users": [12345678901234567u64],
					"authorized_roles": []
				}
			}
		})));
		let serv = controllers_from_bot_data(&data).await;
		serv["foo"].set_error(ServitorError::Unauthorized).await;
		let (author, member) = mock_author_dms(UserId::new(12345678901234567u64));

		let result = action(&data, &serv, "SomeServer", &author, member.as_ref()).await;

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
			&BotData,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
		#[case] calls: (usize, usize, usize, usize, usize),
		#[case] expected_result: T,
	) {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar",
					"authorized_users": [12345678901234567u64],
					"authorized_roles": []
				}
			}
		})));
		let serv = controllers_from_bot_data(&data).await;
		let (author, member) = mock_author_dms(UserId::new(12345678901234567u64));

		let result = action(&data, &serv, "SomeServer", &author, member.as_ref()).await;

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
			&BotData,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
		#[case] calls: (usize, usize, usize, usize, usize),
		#[case] expected_result: T,
	) {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar",
					"authorized_users": [12345678901234567u64],
					"authorized_roles": []
				}
			}
		})));
		let serv = controllers_from_bot_data(&data).await;
		let (author, member) = mock_author_guild(UserId::new(12345678901234567u64), vec![]);

		let result = action(&data, &serv, "SomeServer", &author, member.as_ref()).await;

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
			&BotData,
			&BTreeMap<String, MockServitorController>,
			&str,
			&User,
			Option<&Member>,
		) -> Result<T, ExecuteServitorActionError>,
		#[case] calls: (usize, usize, usize, usize, usize),
		#[case] expected_result: T,
	) {
		let data = mock_data(Some(json!({
			"servitor": {
				"SomeServer": {
					"servitor": "foo",
					"unit_name": "bar",
					"authorized_users": [],
					"authorized_roles": [98765432109876543u64]
				}
			}
		})));
		let serv = controllers_from_bot_data(&data).await;
		let (author, member) = mock_author_guild(
			UserId::new(12345678901234567u64),
			vec![RoleId::new(98765432109876543u64)],
		);

		let result = action(&data, &serv, "SomeServer", &author, member.as_ref()).await;

		assert_eq!(result, Ok(expected_result));
		serv["foo"].assert_called_times(calls.0, calls.1, calls.2, calls.3, calls.4);
	}
}
