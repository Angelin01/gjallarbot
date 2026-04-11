use thiserror::Error;

pub mod authorization;
pub mod machine;
pub mod wake;

#[derive(Debug, Error, PartialEq)]
pub enum MachineError {
	#[error("machine {machine_name} does not exist")]
	DoesNotExist { machine_name: String },

	#[error("machine {machine_name} already exists")]
	AlreadyExists { machine_name: String },
}

#[cfg(test)]
mod tests {
	use crate::db::DbConnection;
	use crate::models::wake_on_lan::NewWakeOnLanMachine;
	use crate::schema::wake_on_lan_machines;
	use diesel_async::RunQueryDsl;

	pub async fn insert_test_machine<D: DbConnection>(conn: &mut D, name: &str, mac: &str) {
		diesel::insert_into(wake_on_lan_machines::table)
			.values(&NewWakeOnLanMachine {
				name,
				mac: &mac.parse().unwrap(),
			})
			.execute(conn)
			.await
			.expect("Failed to insert test machine");
	}
}
