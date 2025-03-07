use crate::controllers::servitor::action::ExecuteServitorActionError;
use crate::embeds;
use crate::services::servitor::{ServitorError, UnitStatus};
use serenity::all::{CreateEmbed, CreateEmbedFooter};

pub fn start_embed(
	result: Result<(), ExecuteServitorActionError>,
	server_name: &str,
) -> CreateEmbed {
	action_embed(result, server_name, "start")
}

pub fn stop_embed(
	result: Result<(), ExecuteServitorActionError>,
	server_name: &str,
) -> CreateEmbed {
	action_embed(result, server_name, "stop")
}

pub fn restart_embed(
	result: Result<(), ExecuteServitorActionError>,
	server_name: &str,
) -> CreateEmbed {
	action_embed(result, server_name, "restart")
}

pub fn reload_embed(
	result: Result<(), ExecuteServitorActionError>,
	server_name: &str,
) -> CreateEmbed {
	action_embed(result, server_name, "reload")
}

pub fn status_embed(
	result: Result<UnitStatus, ExecuteServitorActionError>,
	server_name: &str,
) -> CreateEmbed {
	match result {
		Ok(status) => embeds::success(
			"Servitor server status",
			format!("Status for Servitor server {server_name}"),
		)
		.field("State", status.state, true)
		.field("SubState", status.sub_state, true)
		.footer(CreateEmbedFooter::new("Since:"))
		.timestamp(status.since),
		Err(e) => servitor_error_embed(e, server_name),
	}
}

fn action_embed(
	result: Result<(), ExecuteServitorActionError>,
	server_name: &str,
	action_name: &str,
) -> CreateEmbed {
	match result {
		Ok(_) => embeds::success(
			format!("Servitor server {action_name}"),
			format!("Ran action {action_name} for Servitor server {server_name}"),
		),
		Err(e) => servitor_error_embed(e, server_name),
	}
}

fn servitor_error_embed(error: ExecuteServitorActionError, server_name: &str) -> CreateEmbed {
	match error {
		ExecuteServitorActionError::Server(_) => embeds::invalid_servitor_server(server_name),
		ExecuteServitorActionError::InvalidServitor { servitor_name, .. } => embeds::internal_error(
			"Invalid Servitor",
			format!("The server {server_name} was configured with Servitor {servitor_name}, but that Servitor instance no longer exists. \
				 Please contact the bot owner!")
		),
		ExecuteServitorActionError::Unauthorized { .. } => embeds::error(
			"Unauthorized",
			format!("You are not authorized to operate the Servitor server {server_name}")
		),
		ExecuteServitorActionError::Servitor(se) => match se {
			ServitorError::BadRequest => embeds::internal_error(
				"Bad Request",
				"Received a 'bad request' response from Servitor, either this server is misconfigured or the unit not loaded"
			),
			ServitorError::Unauthorized => embeds::internal_error(
				"Servitor Unauthorized",
				"Failed to authenticate against the Servitor instance, the bot is probably misconfigured! \
					Please contact the bot owner!"
			),
			ServitorError::InternalServerError => embeds::internal_error(
				"Servitor Internal Error",
				"Received an internal server error from Servitor, something is wrong in the server machine"
			),
			ServitorError::Unexpected { .. } => embeds::internal_error(
				"Servitor Unexpected Error",
				"Received an unexpected error while trying to communicate with the Servitor instance, \
					something is really wrong!"
			),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::controllers::servitor::ServerError;
	use crate::services::servitor::ServitorError;
	use chrono::{TimeZone, Utc};
	use reqwest::StatusCode;
	use rstest::rstest;
	use serenity::all::{Colour, CreateEmbedFooter, UserId};
	use std::fmt::Debug;

	#[rstest]
	#[case(start_embed)]
	#[case(stop_embed)]
	#[case(restart_embed)]
	#[case(reload_embed)]
	#[case(status_embed)]
	#[test]
	fn given_action_with_non_existing_server_then_reply_with_invalid_server<
		T: Debug + PartialEq,
	>(
		#[case] function: impl FnOnce(Result<T, ExecuteServitorActionError>, &str) -> CreateEmbed,
	) {
		let result = Err(ExecuteServitorActionError::Server(
			ServerError::DoesNotExist {
				server_name: "NonExistingServer".to_string(),
			},
		));

		let embed = function(result, "NonExistingServer");

		let expected_embed = CreateEmbed::default()
			.title(":x: Invalid Servitor server")
			.colour(Colour(0xdd2e44))
			.description("No Servitor server with name NonExistingServer exists");

		assert_eq!(embed, expected_embed);
	}

	#[rstest]
	#[case(start_embed)]
	#[case(stop_embed)]
	#[case(restart_embed)]
	#[case(reload_embed)]
	#[case(status_embed)]
	#[test]
	fn given_action_with_server_with_invalid_servitor_then_reply_with_invalid_servitor<
		T: Debug + PartialEq,
	>(
		#[case] function: impl FnOnce(Result<T, ExecuteServitorActionError>, &str) -> CreateEmbed,
	) {
		let result = Err(ExecuteServitorActionError::InvalidServitor {
			servitor_name: "foo".to_string(),
			server_name: "SomeServer".to_string(),
		});

		let embed = function(result, "SomeServer");

		let expected_embed = CreateEmbed::default()
			.title(":tools: Invalid Servitor")
			.colour(Colour(0xf4900c))
			.description("The server SomeServer was configured with Servitor foo, but that Servitor instance no longer exists. \
			 Please contact the bot owner!");

		assert_eq!(embed, expected_embed);
	}

	#[rstest]
	#[case(start_embed)]
	#[case(stop_embed)]
	#[case(restart_embed)]
	#[case(reload_embed)]
	#[case(status_embed)]
	#[test]
	fn given_action_with_unauthorized_then_reply_with_unauthorized<T: Debug + PartialEq>(
		#[case] function: impl FnOnce(Result<T, ExecuteServitorActionError>, &str) -> CreateEmbed,
	) {
		let result = Err(ExecuteServitorActionError::Unauthorized {
			user: UserId::new(12345678901234567),
			server_name: "SomeServer".to_string(),
		});

		let embed = function(result, "SomeServer");

		let expected_embed = CreateEmbed::default()
			.title(":x: Unauthorized")
			.colour(Colour(0xdd2e44))
			.description("You are not authorized to operate the Servitor server SomeServer");

		assert_eq!(embed, expected_embed);
	}

	#[rstest]
	#[case(start_embed)]
	#[case(stop_embed)]
	#[case(restart_embed)]
	#[case(reload_embed)]
	#[case(status_embed)]
	#[test]
	fn given_action_with_servitor_bad_request_then_reply_with_bad_request<T: Debug + PartialEq>(
		#[case] function: impl FnOnce(Result<T, ExecuteServitorActionError>, &str) -> CreateEmbed,
	) {
		let result = Err(ExecuteServitorActionError::Servitor(
			ServitorError::BadRequest,
		));

		let embed = function(result, "SomeServer");

		let expected_embed = CreateEmbed::default()
			.title(":tools: Bad Request")
			.colour(Colour(0xf4900c))
			.description("Received a 'bad request' response from Servitor, either this server is misconfigured or the unit not loaded");

		assert_eq!(embed, expected_embed);
	}

	#[rstest]
	#[case(start_embed)]
	#[case(stop_embed)]
	#[case(restart_embed)]
	#[case(reload_embed)]
	#[case(status_embed)]
	#[test]
	fn given_action_with_servitor_unauthorized_then_reply_with_servitor_misconfigured<
		T: Debug + PartialEq,
	>(
		#[case] function: impl FnOnce(Result<T, ExecuteServitorActionError>, &str) -> CreateEmbed,
	) {
		let result = Err(ExecuteServitorActionError::Servitor(
			ServitorError::Unauthorized,
		));

		let embed = function(result, "SomeServer");

		let expected_embed = CreateEmbed::default()
			.title(":tools: Servitor Unauthorized")
			.colour(Colour(0xf4900c))
			.description("Failed to authenticate against the Servitor instance, the bot is probably misconfigured! \
			Please contact the bot owner!");

		assert_eq!(embed, expected_embed);
	}

	#[rstest]
	#[case(start_embed)]
	#[case(stop_embed)]
	#[case(restart_embed)]
	#[case(reload_embed)]
	#[case(status_embed)]
	#[test]
	fn given_action_with_servitor_interalservererror_then_reply_with_servitor_internalservererror<
		T: Debug + PartialEq,
	>(
		#[case] function: impl FnOnce(Result<T, ExecuteServitorActionError>, &str) -> CreateEmbed,
	) {
		let result = Err(ExecuteServitorActionError::Servitor(
			ServitorError::InternalServerError,
		));

		let embed = function(result, "SomeServer");

		let expected_embed = CreateEmbed::default()
			.title(":tools: Servitor Internal Error")
			.colour(Colour(0xf4900c))
			.description("Received an internal server error from Servitor, something is wrong in the server machine");

		assert_eq!(embed, expected_embed);
	}

	#[rstest]
	#[case(start_embed)]
	#[case(stop_embed)]
	#[case(restart_embed)]
	#[case(reload_embed)]
	#[case(status_embed)]
	#[test]
	fn given_action_with_servitor_unexpected_then_reply_with_servitor_unexpected<
		T: Debug + PartialEq,
	>(
		#[case] function: impl FnOnce(Result<T, ExecuteServitorActionError>, &str) -> CreateEmbed,
	) {
		let result = Err(ExecuteServitorActionError::Servitor(
			ServitorError::Unexpected {
				status_code: Some(StatusCode::IM_A_TEAPOT),
				error: None,
			},
		));

		let embed = function(result, "SomeServer");

		let expected_embed = CreateEmbed::default()
			.title(":tools: Servitor Unexpected Error")
			.colour(Colour(0xf4900c))
			.description("Received an unexpected error while trying to communicate with the Servitor instance, \
			something is really wrong!");

		assert_eq!(embed, expected_embed);
	}

	#[rstest]
	#[case(start_embed, "start")]
	#[case(stop_embed, "stop")]
	#[case(restart_embed, "restart")]
	#[case(reload_embed, "reload")]
	#[test]
	fn given_action_with_success_then_reply_with_success_info(
		#[case] function: impl FnOnce(Result<(), ExecuteServitorActionError>, &str) -> CreateEmbed,
		#[case] action_name: &str,
	) {
		let embed = function(Ok(()), "SomeServer");

		let expected_embed = CreateEmbed::default()
			.title(format!(":white_check_mark: Servitor server {action_name}"))
			.colour(Colour(0x77b255))
			.description(format!(
				"Ran action {action_name} for Servitor server SomeServer"
			));

		assert_eq!(embed, expected_embed);
	}

	#[test]
	fn given_status_success_then_reply_with_status_info() {
		let result = Ok(UnitStatus {
			service: "bar.service".to_string(),
			state: "active".to_string(),
			sub_state: "running".to_string(),
			since: Utc.with_ymd_and_hms(2025, 3, 6, 19, 59, 45).unwrap(),
		});

		let embed = status_embed(result, "SomeServer");

		let expected_embed = CreateEmbed::default()
			.title(":white_check_mark: Servitor server status")
			.colour(Colour(0x77b255))
			.description("Status for Servitor server SomeServer")
			.field("State", "active", true)
			.field("SubState", "running", true)
			.footer(CreateEmbedFooter::new("Since:"))
			.timestamp(Utc.with_ymd_and_hms(2025, 3, 6, 19, 59, 45).unwrap());

		assert_eq!(embed, expected_embed);
	}
}
