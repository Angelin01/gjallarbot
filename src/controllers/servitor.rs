use crate::data::servitor::ServerInfo;
use crate::data::{Data, PersistentJson, PersistentWriteGuard};
use thiserror::Error;
use tokio::sync::RwLockReadGuard;

pub mod authorization;
pub mod server;

#[derive(Debug, Error, PartialEq)]
pub enum ServerError {
	#[error("server {server_name} does not exist")]
	DoesNotExist { server_name: String },

	#[error("server {server_name} already exists")]
	AlreadyExists { server_name: String },
}

async fn get_server_info_mut<'a>(
	data_write: &'a mut PersistentWriteGuard<'_, Data>,
	server_name: &str,
) -> Result<&'a mut ServerInfo, ServerError> {
	data_write
		.servitor
		.get_mut(server_name)
		.ok_or(ServerError::DoesNotExist {
			server_name: server_name.into(),
		})
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
