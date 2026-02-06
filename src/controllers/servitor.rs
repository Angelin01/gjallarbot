use thiserror::Error;

pub mod authorization;
pub mod server;
pub mod action;

#[derive(Debug, Error, PartialEq)]
pub enum ServerError {
	#[error("server {server_name} does not exist")]
	DoesNotExist { server_name: String },

	#[error("server {server_name} already exists")]
	AlreadyExists { server_name: String },
}
