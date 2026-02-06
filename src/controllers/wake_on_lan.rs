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
