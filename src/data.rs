mod persistent_data;

use crate::models::servitor::*;
use crate::schema::servitor_servers::dsl::*;
use anyhow::Result;
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use servitor::ServitorData;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use wake_on_lan::WakeOnLanData;

pub mod authorization;
pub mod servitor;
pub mod wake_on_lan;

use crate::db::DbConnection;
use crate::models::wake_on_lan::{
	NewWakeOnLanMachine, NewWakeOnLanMachineAuthorizedRole, NewWakeOnLanMachineAuthorizedUser,
	WakeOnLanMachine,
};
use crate::schema::servitor_server_authorized_roles::dsl::servitor_server_authorized_roles;
use crate::schema::servitor_server_authorized_users::dsl::servitor_server_authorized_users;
use crate::schema::wake_on_lan_machines::dsl::wake_on_lan_machines;
use crate::schema::wake_on_lan_machines_authorized_roles::dsl::wake_on_lan_machines_authorized_roles;
use crate::schema::wake_on_lan_machines_authorized_users::dsl::wake_on_lan_machines_authorized_users;
pub use persistent_data::*;

#[derive(Deserialize, Serialize, Default)]
pub struct Data {
	#[serde(default)]
	pub wake_on_lan: WakeOnLanData,
	#[serde(default)]
	pub servitor: ServitorData,
}

pub type BotData = Arc<RwLock<PersistentJson<Data>>>;

pub async fn migrate_old_data<C>(path: impl AsRef<str>, conn: &mut C) -> Result<()>
where
	C: DbConnection,
{
	let old_data_path = PathBuf::from(path.as_ref());

	if !(old_data_path.exists() && old_data_path.is_file()) {
		return Ok(());
	}

	let old_data = PersistentJson::<Data>::new(path.as_ref())?;
	conn.transaction::<_, anyhow::Error, _>(|conn| {
		Box::pin(async move {
			migrate_wake_on_lan(&old_data.wake_on_lan, conn).await?;
			migrate_servitor(&old_data.servitor, conn).await?;
			Ok(())
		})
	})
	.await

	// TODO: delete old file
}

async fn migrate_wake_on_lan<C>(wake_on_lan_data: &WakeOnLanData, conn: &mut C) -> Result<()>
where
	C: DbConnection,
{
	for (machine_name, machine_info) in wake_on_lan_data {
		let new_machine = NewWakeOnLanMachine {
			name: machine_name,
			mac: &machine_info.mac,
		};

		let machine = diesel::insert_into(wake_on_lan_machines)
			.values(new_machine)
			.returning(WakeOnLanMachine::as_returning())
			.get_result(&mut *conn)
			.await?;

		let new_users: Vec<NewWakeOnLanMachineAuthorizedUser> = machine_info
			.authorized_users
			.iter()
			.map(|&user_id| NewWakeOnLanMachineAuthorizedUser {
				machine_id: machine.id,
				user_id: user_id.get() as i64,
			})
			.collect();

		for new_user in new_users {
			diesel::insert_into(wake_on_lan_machines_authorized_users)
				.values(new_user)
				.execute(&mut *conn)
				.await?;
		}

		let new_roles: Vec<NewWakeOnLanMachineAuthorizedRole> = machine_info
			.authorized_roles
			.iter()
			.map(|&role_id| NewWakeOnLanMachineAuthorizedRole {
				machine_id: machine.id,
				role_id: role_id.get() as i64,
			})
			.collect();

		for new_role in new_roles {
			diesel::insert_into(wake_on_lan_machines_authorized_roles)
				.values(new_role)
				.execute(&mut *conn)
				.await?;
		}
	}

	Ok(())
}

async fn migrate_servitor<C>(servitor_data: &ServitorData, conn: &mut C) -> Result<()>
where
	C: DbConnection,
{
	for (server_name, server_info) in servitor_data {
		let new_server = NewServitorServer {
			name: server_name,
			servitor: server_info.servitor.as_str(),
			unit_name: server_info.unit_name.as_str(),
		};

		let server = diesel::insert_into(servitor_servers)
			.values(&new_server)
			.returning(ServitorServer::as_returning())
			.get_result(&mut *conn)
			.await?;

		let new_users: Vec<NewServitorServerAuthorizedUser> = server_info
			.authorized_users
			.iter()
			.map(|&user_id| NewServitorServerAuthorizedUser {
				server_id: server.id,
				user_id: user_id.get() as i64,
			})
			.collect();

		for new_user in &new_users {
			diesel::insert_into(servitor_server_authorized_users)
				.values(new_user)
				.execute(&mut *conn)
				.await?;
		}

		let new_roles: Vec<NewServitorServerAuthorizedRole> = server_info
			.authorized_roles
			.iter()
			.map(|&role_id| NewServitorServerAuthorizedRole {
				server_id: server.id,
				role_id: role_id.get() as i64,
			})
			.collect();

		for new_role in &new_roles {
			diesel::insert_into(servitor_server_authorized_roles)
				.values(new_role)
				.execute(&mut *conn)
				.await?;
		}
	}

	Ok(())
}

#[cfg(test)]
pub mod tests {
	use super::*;
	use serde_json::Value;
	use std::io::Write;
	use tempfile::NamedTempFile;
	pub fn mock_data(initial_data: Option<Value>) -> BotData {
		let mut temp_file = NamedTempFile::new().unwrap();
		if let Some(data) = initial_data {
			temp_file
				.write_all(serde_json::to_string(&data).unwrap().as_bytes())
				.unwrap();
			temp_file.flush().unwrap();
		}

		let persistent_data = PersistentJson::new(temp_file.path()).unwrap();

		Arc::new(RwLock::new(persistent_data))
	}
}
