use thiserror::Error;
use tokio::sync::RwLockReadGuard;
use crate::data::{Data, PersistentJson, PersistentWriteGuard};
use crate::data::wake_on_lan::WakeOnLanMachineInfo;

mod authorization;
mod machine;
mod wake;

#[derive(Debug, Error, PartialEq)]
pub enum MachineError {
	#[error("machine {machine_name} does not exist")]
	DoesNotExist { machine_name: String },

	#[error("machine {machine_name} already exists")]
	AlreadyExists { machine_name: String },
}

async fn get_machine_info_mut<'a>(
	data_write: &'a mut PersistentWriteGuard<'_, Data>,
	machine_name: &str,
) -> Result<&'a mut WakeOnLanMachineInfo, MachineError> {
	data_write
		.wake_on_lan
		.get_mut(machine_name)
		.ok_or(MachineError::DoesNotExist {
			machine_name: machine_name.into(),
		})
}

async fn get_machine_info<'a>(
	data_read: &'a RwLockReadGuard<'_, PersistentJson<Data>>,
	machine_name: &str,
) -> Result<&'a WakeOnLanMachineInfo, MachineError> {
	data_read
		.wake_on_lan
		.get(machine_name)
		.ok_or(MachineError::DoesNotExist {
			machine_name: machine_name.into(),
		})
}
