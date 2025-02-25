pub trait ServitorController = ServitorHandler + Send + Sync;
pub trait ServitorHandler {
	async fn start(&self, unit_name: &str);
	async fn stop(&self, unit_name: &str);
	async fn restart(&self, unit_name: &str);
	async fn reload(&self, unit_name: &str);
	async fn status(&self, unit_name: &str);
}

pub struct HttpServitorController;

impl ServitorHandler for HttpServitorController {
	async fn start(&self, unit_name: &str) {
		todo!()
	}

	async fn stop(&self, unit_name: &str) {
		todo!()
	}

	async fn restart(&self, unit_name: &str) {
		todo!()
	}

	async fn reload(&self, unit_name: &str) {
		todo!()
	}

	async fn status(&self, unit_name: &str) {
		todo!()
	}
}

#[cfg(test)]
pub mod tests {
	use super::*;
	use crate::data::BotData;
	use std::collections::BTreeMap;

	pub async fn controllers_from_bot_data(
		data: &BotData,
	) -> BTreeMap<String, MockServitorController> {
		data.read()
			.await
			.servitor
			.iter()
			.map(|(_, server_info)| (server_info.servitor.clone(), MockServitorController::new()))
			.collect()
	}

	pub struct MockServitorController {
		called: bool,
	}

	impl MockServitorController {
		pub fn new() -> Self {
			Self { called: false }
		}

		pub fn assert_not_called(&self) { assert!(!self.called) }
	}

	impl ServitorHandler for MockServitorController {
		async fn start(&self, unit_name: &str) {
			todo!()
		}

		async fn stop(&self, unit_name: &str) {
			todo!()
		}

		async fn restart(&self, unit_name: &str) {
			todo!()
		}

		async fn reload(&self, unit_name: &str) {
			todo!()
		}

		async fn status(&self, unit_name: &str) {
			todo!()
		}
	}
}
