use thiserror::Error;

pub mod action;
pub mod authorization;
pub mod server;

#[derive(Debug, Error, PartialEq)]
pub enum ServerError {
	#[error("server {server_name} does not exist")]
	DoesNotExist { server_name: String },

	#[error("server {server_name} already exists")]
	AlreadyExists { server_name: String },
}

#[cfg(test)]
mod tests {
	use crate::db::DbConnection;
	use crate::models::servitor::NewServitorServer;
	use crate::schema::servitor_servers;
	use diesel_async::RunQueryDsl;

	pub async fn insert_test_server<D: DbConnection>(
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
}
