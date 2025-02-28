use crate::data::servitor::ServerInfo;
use crate::data::{Data, PersistentJson};
use thiserror::Error;
use tokio::sync::RwLockReadGuard;

pub mod server;

#[derive(Debug, Error, PartialEq)]
pub enum ServerError {
	#[error("server {server_name} does not exist")]
	DoesNotExist { server_name: String },

	#[error("server {server_name} already exists")]
	AlreadyExists { server_name: String },
}

async fn get_server_info<'a>(
	data_read: &'a RwLockReadGuard<'_, PersistentJson<Data>>,
	server_name: &str,
) -> Result<&'a ServerInfo, ServerError> {
	data_read
		.servitor
		.get(server_name)
		.ok_or(ServerError::DoesNotExist {
			server_name: server_name.into(),
		})
}
