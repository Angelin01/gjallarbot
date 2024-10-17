use anyhow::Result;
use log::error;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

pub trait PersistentData: Serialize + for<'de> Deserialize<'de> {}
impl<T: Serialize + for<'de> Deserialize<'de>> PersistentData for T {}

pub struct PersistentWriteGuard<'a, T: PersistentData> {
	data: &'a mut T,
	path: &'a Path,
}

impl<'a, T: PersistentData> PersistentWriteGuard<'a, T> {
	fn write_to_file(&self) -> Result<()> {
		let json = serde_json::to_string(&*self.data)?;
		let mut file = File::create(self.path)?;
		file.write_all(json.as_bytes())?;
		Ok(())
	}
}

impl<'a, T: PersistentData> PersistentWriteGuard<'a, T> {
	pub fn new(data: &'a mut T, path: &'a Path) -> Self {
		Self { data, path }
	}
}

impl<'a, T: PersistentData> Deref for PersistentWriteGuard<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<'a, T: PersistentData> DerefMut for PersistentWriteGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.data
	}
}

impl<'a, T: PersistentData> Drop for PersistentWriteGuard<'a, T> {
	fn drop(&mut self) {
		if let Err(e) = self.write_to_file() {
			error!("Failed to write persistent data to file {}: {}", self.path.display(), e);
		}
	}
}

pub struct PersistentJson<T: PersistentData> {
	data: T,
	path: PathBuf,
}

impl<T: PersistentData + Default> PersistentJson<T> {
	pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
		let data = if fs::exists(&path)? {
			let content = fs::read_to_string(&path)?;
			serde_json::from_str(&content)?
		} else {
			Default::default()
		};

		Ok(Self {
			data,
			path: path.as_ref().to_path_buf(),
		})
	}

	pub fn write(&mut self) -> PersistentWriteGuard<T> {
		PersistentWriteGuard::new(&mut self.data, &self.path)
	}
}

impl<T: PersistentData>  Deref for PersistentJson<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde::{Deserialize, Serialize};
	use serde_json::json;
	use tempfile::tempdir;

	#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
	struct TestConfig {
		setting1: String,
		setting2: u32,
	}

	#[test]
	fn given_file_doesnt_exist_then_should_use_default() {
		let dir = tempdir().unwrap();
		let file_path = dir.path().join("test_config.json");

		let persistent_json: PersistentJson<TestConfig> = PersistentJson::new(&file_path).unwrap();

		assert_eq!(&*persistent_json, &TestConfig::default());
	}

	#[test]
	fn given_file_doesnt_exist_then_should_write_it() {
		let dir = tempdir().unwrap();
		let file_path = dir.path().join("test_config.json");

		let mut persistent_json: PersistentJson<TestConfig> = PersistentJson::new(&file_path).unwrap();
		{
			let mut write_guard = persistent_json.write();
			write_guard.setting1 = "value1".to_string();
			write_guard.setting2 = 42;
		}

		let content = fs::read_to_string(&file_path).unwrap();
		let deserialized_data: TestConfig = serde_json::from_str(&content).unwrap();

		let expected_config = TestConfig {
			setting1: "value1".to_string(),
			setting2: 42,
		};

		assert_eq!(deserialized_data, expected_config);
	}

	#[test]
	fn given_existing_file_then_should_read_it() {
		let dir = tempdir().unwrap();
		let file_path = dir.path().join("test_config.json");

		let json_data = json!({
			"setting1": "value1",
			"setting2": 42
		});

		let mut file = File::create(&file_path).unwrap();
		file.write_all(serde_json::to_string(&json_data).unwrap().as_bytes()).unwrap();

		let persistent_json: PersistentJson<TestConfig> = PersistentJson::new(&file_path).unwrap();

		let expected_config = TestConfig {
			setting1: "value1".to_string(),
			setting2: 42,
		};
		assert_eq!(&*persistent_json, &expected_config);
	}

	#[test]
	fn given_existing_file_and_updated_content_then_should_update_file() {
		let dir = tempdir().unwrap();
		let file_path = dir.path().join("test_config.json");

		let json_data = json!({
			"setting1": "value1",
			"setting2": 42
		});

		let mut file = File::create(&file_path).unwrap();
		file.write_all(serde_json::to_string(&json_data).unwrap().as_bytes()).unwrap();

		let mut persistent_json: PersistentJson<TestConfig> = PersistentJson::new(&file_path).unwrap();
		{
			let mut write_guard = persistent_json.write();
			write_guard.setting1 = "other".to_string();
			write_guard.setting2 = 69;
		}

		let content = fs::read_to_string(&file_path).unwrap();
		let deserialized_data: TestConfig = serde_json::from_str(&content).unwrap();

		let expected_config = TestConfig {
			setting1: "other".to_string(),
			setting2: 69,
		};

		assert_eq!(deserialized_data, expected_config);
	}
}
