mod persistent_data;

use serde::{Deserialize, Serialize};
use servitor::ServitorData;
use std::sync::Arc;
use tokio::sync::RwLock;
use wake_on_lan::WakeOnLanData;

pub mod servitor;
pub mod wake_on_lan;

pub use persistent_data::*;

#[derive(Deserialize, Serialize, Default)]
pub struct Data {
	#[serde(default)]
	pub wake_on_lan: WakeOnLanData,
	#[serde(default)]
	pub servitor: ServitorData,
}

pub type BotData = Arc<RwLock<PersistentJson<Data>>>;

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
