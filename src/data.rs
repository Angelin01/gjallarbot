mod persistent_data;

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use wake_on_lan::WakeOnLanData;

pub mod wake_on_lan;
pub use persistent_data::*;

#[derive(Deserialize, Serialize, Default)]
pub struct Data {
    pub wake_on_lan: WakeOnLanData,
}

pub type BotData = Arc<RwLock<PersistentJson<Data>>>;
pub type BotError = Box<dyn std::error::Error + Send + Sync>;

pub type Context<'a> = poise::Context<'a, BotData, BotError>;

#[cfg(test)]
pub mod tests {
	use super::*;
	use serde_json::Value;
	use std::io::Write;
	use tempfile::NamedTempFile;
	pub fn mock_data(initial_data: Option<Value>) -> BotData {
		let mut temp_file = NamedTempFile::new().unwrap();
		if let Some(data) = initial_data {
			temp_file.write_all(serde_json::to_string(&data).unwrap().as_bytes()).unwrap();
			temp_file.flush().unwrap();
		}

		let persistent_data = PersistentJson::new(temp_file.path()).unwrap();

		Arc::new(RwLock::new(persistent_data))
	}
}
