use super::MachineError;
use crate::db::DbConnection;
use crate::errors::UnexpectedError;
use crate::models::wake_on_lan::{
	WakeOnLanMachine, WakeOnLanMachineAuthorizedRole, WakeOnLanMachineAuthorizedUser,
};
use crate::schema::{
	wake_on_lan_machines, wake_on_lan_machines_authorized_roles,
	wake_on_lan_machines_authorized_users,
};
use crate::services::wake_on_lan::{MagicPacket, MagicPacketSender};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serenity::all::{Member, User, UserId};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum WakeError {
	#[error(transparent)]
	Machine(#[from] MachineError),

	#[error("Error sending wake command: {kind:?}")]
	Io { kind: std::io::ErrorKind },

	#[error("User {user} is not authorized to wake up machine {machine_name}")]
	Unauthorized { user: UserId, machine_name: String },

	#[error("An unexpected error occurred")]
	Unexpected(#[from] UnexpectedError),
}

pub async fn wake<D: DbConnection, S: MagicPacketSender>(
	conn: &mut D,
	author: &User,
	member: Option<&Member>,
	machine_name: &str,
	sender: &S,
) -> Result<(), WakeError> {
	let machine: WakeOnLanMachine = wake_on_lan_machines::table
		.filter(wake_on_lan_machines::name.eq(machine_name))
		.first(conn)
		.await
		.map_err(|e| match e {
			diesel::result::Error::NotFound => MachineError::DoesNotExist {
				machine_name: machine_name.into(),
			}
			.into(),
			other => WakeError::Unexpected(UnexpectedError(
				anyhow::Error::new(other).context("Failed to query machine from database"),
			)),
		})?;

	let mut authorized = wake_on_lan_machines_authorized_users::table
		.filter(wake_on_lan_machines_authorized_users::machine_id.eq(machine.id))
		.filter(wake_on_lan_machines_authorized_users::user_id.eq(author.id.get() as i64))
		.first::<WakeOnLanMachineAuthorizedUser>(conn)
		.await
		.is_ok();

	if !authorized {
		if let Some(m) = member {
			let role_ids: Vec<i64> = m.roles.iter().map(|r| r.get() as i64).collect();
			authorized = wake_on_lan_machines_authorized_roles::table
				.filter(wake_on_lan_machines_authorized_roles::machine_id.eq(machine.id))
				.filter(wake_on_lan_machines_authorized_roles::role_id.eq_any(role_ids))
				.first::<WakeOnLanMachineAuthorizedRole>(conn)
				.await
				.is_ok();
		}
	}

	if !authorized {
		return Err(WakeError::Unauthorized {
			user: author.id,
			machine_name: machine_name.to_string(),
		});
	}

	sender
		.send(&MagicPacket::from_mac(&machine.mac))
		.await
		.map_err(|e| WakeError::Io { kind: e.kind() })
}

#[cfg(test)]
mod tests {
	use super::super::super::tests::{mock_author_dms, mock_author_guild};
	use super::*;
	use crate::controllers::wake_on_lan::tests::insert_test_machine;
	use crate::db::tests::setup_test_db;
	use crate::models::wake_on_lan::{
		NewWakeOnLanMachineAuthorizedRole, NewWakeOnLanMachineAuthorizedUser,
	};
	use crate::schema::wake_on_lan_machines;
	use crate::services::wake_on_lan::MacAddress;
	use serenity::all::RoleId;
	use std::cell::Cell;

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

	async fn get_machine_id<D: DbConnection>(conn: &mut D, name: &str) -> i32 {
		wake_on_lan_machines::table
			.filter(wake_on_lan_machines::name.eq(name))
			.select(wake_on_lan_machines::id)
			.first::<i32>(conn)
			.await
			.expect("Failed to get machine id")
	}

	async fn permit_test_user<D: DbConnection>(conn: &mut D, machine_name: &str, user_id: u64) {
		let machine_id = get_machine_id(conn, machine_name).await;
		diesel::insert_into(wake_on_lan_machines_authorized_users::table)
			.values(&NewWakeOnLanMachineAuthorizedUser {
				machine_id,
				user_id: user_id as i64,
			})
			.execute(conn)
			.await
			.expect("Failed to insert authorized user");
	}

	async fn permit_test_role<D: DbConnection>(conn: &mut D, machine_name: &str, role_id: u64) {
		let machine_id = get_machine_id(conn, machine_name).await;
		diesel::insert_into(wake_on_lan_machines_authorized_roles::table)
			.values(&NewWakeOnLanMachineAuthorizedRole {
				machine_id,
				role_id: role_id as i64,
			})
			.execute(conn)
			.await
			.expect("Failed to insert authorized role");
	}

	#[tokio::test]
	async fn given_nonexistent_machine_then_wake_returns_error() {
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;

		let sender = MockMagicPacketSender::default();
		let (author, member) = mock_author_dms(UserId::new(12345678901234567));

		let result = wake(
			&mut conn,
			&author,
			member.as_ref(),
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
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;
		permit_test_user(&mut conn, "ExistingMachine", 98765432109876543).await;
		permit_test_user(&mut conn, "ExistingMachine", 98765432109876542).await;
		permit_test_role(&mut conn, "ExistingMachine", 98765432109876541).await;
		permit_test_role(&mut conn, "ExistingMachine", 98765432109876540).await;

		let sender = MockMagicPacketSender::default();
		let (author, member) = mock_author_dms(UserId::new(12345678901234567));

		let result = wake(
			&mut conn,
			&author,
			member.as_ref(),
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
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;
		permit_test_user(&mut conn, "ExistingMachine", 98765432109876543).await;
		permit_test_user(&mut conn, "ExistingMachine", 98765432109876542).await;
		permit_test_role(&mut conn, "ExistingMachine", 98765432109876541).await;
		permit_test_role(&mut conn, "ExistingMachine", 98765432109876540).await;

		let sender = MockMagicPacketSender::default();
		let (author, member) = mock_author_guild(
			UserId::new(12345678901234567),
			vec![RoleId::new(12345678901234567)],
		);

		let result = wake(
			&mut conn,
			&author,
			member.as_ref(),
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
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;
		permit_test_user(&mut conn, "ExistingMachine", 12345678901234567).await;

		let sender = MockMagicPacketSender::default();
		let (author, member) = mock_author_dms(UserId::new(12345678901234567));

		let result = wake(
			&mut conn,
			&author,
			member.as_ref(),
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
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;
		permit_test_user(&mut conn, "ExistingMachine", 12345678901234567).await;

		let sender = MockMagicPacketSender::default();
		let (author, member) = mock_author_guild(
			UserId::new(12345678901234567),
			vec![RoleId::new(98765432109876543)],
		);

		let result = wake(
			&mut conn,
			&author,
			member.as_ref(),
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
		let mut conn = setup_test_db().await;
		insert_test_machine(&mut conn, "ExistingMachine", "01:02:03:04:05:06").await;
		permit_test_role(&mut conn, "ExistingMachine", 98765432109876543).await;

		let sender = MockMagicPacketSender::default();
		let (author, member) = mock_author_guild(
			UserId::new(12345678901234567),
			vec![RoleId::new(98765432109876543)],
		);

		let result = wake(
			&mut conn,
			&author,
			member.as_ref(),
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
