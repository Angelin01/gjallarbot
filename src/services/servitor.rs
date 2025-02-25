
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
