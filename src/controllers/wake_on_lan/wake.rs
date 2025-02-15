use super::{get_machine_info, MachineError};
use crate::data::wake_on_lan::WakeOnLanMachineInfo;
use crate::data::BotData;
use crate::services::wake_on_lan::{MagicPacket, MagicPacketSender};
use serenity::all::{User, UserId};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum WakeError {
	#[error(transparent)]
	Machine(#[from] MachineError),

	#[error("Error sending wake command: {kind:?}")]
	Io { kind: std::io::ErrorKind },

	#[error("User {user} is not authorized to wake up machine {machine_name}")]
	Unauthorized { user: UserId, machine_name: String },
}

async fn wake<S: MagicPacketSender>(
	data: &BotData,
	author: &User,
	machine_name: &str,
	sender: &S,
) -> Result<(), WakeError> {
	let data_read = data.read().await;

	let machine_info = get_machine_info(&data_read, machine_name).await?;

	if !is_user_authorized(author, &machine_info) {
		return Err(WakeError::Unauthorized {
			user: author.id.to_owned(),
			machine_name: machine_name.to_string(),
		});
	}

	sender
		.send(&MagicPacket::from_mac(&machine_info.mac))
		.await
		.map_err(|e| WakeError::Io { kind: e.kind() })
}

fn is_user_authorized(author: &User, machine_info: &WakeOnLanMachineInfo) -> bool {
	machine_info.authorized_users.contains(&author.id)
		|| author.member.as_ref().map_or(false, |m| {
			m.roles
				.iter()
				.any(|&role| machine_info.authorized_roles.contains(&role))
		})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::data::tests::mock_data;
	use crate::services::wake_on_lan::MacAddress;
	use serde_json::json;
	use serenity::all::{Member, RoleId};
	use std::cell::Cell;

	fn mock_author_dms(id: UserId) -> User {
		let mut user = User::default();
		user.id = id;
		user.name = "mock_author".to_string();
		user
	}

	fn mock_author_guild(id: UserId, roles: Vec<RoleId>) -> User {
		let mut user = mock_author_dms(id);

		let mut member = Member::default();
		member.roles = roles;

		user.member = Some(Box::new(member.into()));

		user
	}

	#[derive(Default)]
	struct MockMagicPacketSender {
		sent_magic_packet: Cell<Option<MagicPacket>>,
	}

	impl MockMagicPacketSender {
		pub fn assert_no_packet_sent(&self) {
			assert_eq!(self.sent_magic_packet.take(), None);
		}

		pub fn assert_packet_sent(&self, expected_packet: &MagicPacket) {
			let sent_packet = self.sent_magic_packet.take();

			assert_eq!(sent_packet.as_ref(), Some(expected_packet));

			self.sent_magic_packet.set(sent_packet);
		}
	}

	impl MagicPacketSender for MockMagicPacketSender {
		async fn send(&self, magic_packet: &MagicPacket) -> std::io::Result<()> {
			self.sent_magic_packet.set(Some(magic_packet.clone()));

			Ok(())
		}
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_wake_returns_error() {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6]
				}
			}
		})));

		let sender = MockMagicPacketSender::default();

		let result = wake(
			&data,
			&mock_author_dms(UserId::new(12345678901234567)),
			"NonexistentMachine",
			&sender,
		)
		.await;

		assert_eq!(
			result,
			Err(WakeError::Machine(MachineError::DoesNotExist {
				machine_name: "NonexistentMachine".to_owned(),
			}))
		);
		sender.assert_no_packet_sent();
	}

	#[tokio::test]
	async fn given_dm_call_and_existing_machine_but_user_not_in_allowed_list_then_wake_returns_error(
	) {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [98765432109876543i64, 98765432109876542i64],
					"authorized_roles": [98765432109876541i64, 98765432109876540i64]
				}
			}
		})));
		let sender = MockMagicPacketSender::default();

		let result = wake(
			&data,
			&mock_author_dms(UserId::new(12345678901234567)),
			"ExistingMachine",
			&sender,
		)
		.await;

		assert_eq!(
			result,
			Err(WakeError::Unauthorized {
				user: UserId::new(12345678901234567),
				machine_name: "ExistingMachine".to_owned(),
			})
		);
		sender.assert_no_packet_sent();
	}

	#[tokio::test]
	async fn given_guild_call_and_existing_machine_but_user_not_in_allowed_list_then_wake_returns_error(
	) {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [98765432109876543i64, 98765432109876542i64],
					"authorized_roles": [98765432109876541i64, 98765432109876540i64]
				}
			}
		})));
		let sender = MockMagicPacketSender::default();

		let result = wake(
			&data,
			&mock_author_guild(
				UserId::new(12345678901234567),
				vec![RoleId::new(12345678901234567)],
			),
			"ExistingMachine",
			&sender,
		)
		.await;

		assert_eq!(
			result,
			Err(WakeError::Unauthorized {
				user: UserId::new(12345678901234567),
				machine_name: "ExistingMachine".to_owned(),
			})
		);
		sender.assert_no_packet_sent();
	}

	#[tokio::test]
	async fn given_dm_call_and_existing_machine_and_user_in_allowed_list_then_should_wake_machine()
	{
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [12345678901234567i64]
				}
			}
		})));
		let sender = MockMagicPacketSender::default();

		let result = wake(
			&data,
			&mock_author_dms(UserId::new(12345678901234567)),
			"ExistingMachine",
			&sender,
		)
		.await;

		let expected_magic_packet =
			MagicPacket::from_mac(&MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]));

		assert_eq!(result, Ok(()));
		sender.assert_packet_sent(&expected_magic_packet);
	}

	#[tokio::test]
	async fn given_guild_call_and_existing_machine_and_user_in_allowed_list_then_should_wake_machine(
	) {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [12345678901234567i64],
					"authorized_roles": []
				}
			}
		})));
		let sender = MockMagicPacketSender::default();

		let result = wake(
			&data,
			&mock_author_guild(
				UserId::new(12345678901234567),
				vec![RoleId::new(98765432109876543)],
			),
			"ExistingMachine",
			&sender,
		)
		.await;

		let expected_magic_packet =
			MagicPacket::from_mac(&MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]));

		assert_eq!(result, Ok(()));
		sender.assert_packet_sent(&expected_magic_packet);
	}

	#[tokio::test]
	async fn given_guild_call_and_existing_machine_and_user_in_allowed_roles_then_should_wake_machine(
	) {
		let data = mock_data(Some(json!({
			"wake_on_lan": {
				"ExistingMachine": {
					"mac": [1, 2, 3, 4, 5, 6],
					"authorized_users": [],
					"authorized_roles": [98765432109876543i64]
				}
			}
		})));
		let sender = MockMagicPacketSender::default();

		let result = wake(
			&data,
			&mock_author_guild(
				UserId::new(12345678901234567),
				vec![RoleId::new(98765432109876543)],
			),
			"ExistingMachine",
			&sender,
		)
		.await;

		let expected_magic_packet =
			MagicPacket::from_mac(&MacAddress([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]));

		assert_eq!(result, Ok(()));
		sender.assert_packet_sent(&expected_magic_packet);
	}
}
