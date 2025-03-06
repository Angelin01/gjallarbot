use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Client, Response, StatusCode};
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use thiserror::Error;

pub trait ServitorController = ServitorHandler + Send + Sync;
pub trait ServitorHandler {
	async fn start(&self, unit_name: &str) -> Result<(), ServitorError>;
	async fn stop(&self, unit_name: &str) -> Result<(), ServitorError>;
	async fn restart(&self, unit_name: &str) -> Result<(), ServitorError>;
	async fn reload(&self, unit_name: &str) -> Result<(), ServitorError>;
	async fn status(&self, unit_name: &str) -> Result<UnitStatus, ServitorError>;
	async fn health(&self) -> bool;
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UnitStatus {
	pub service: String,
	pub state: String,
	pub sub_state: String,
	pub since: DateTime<Utc>,
}

pub struct HttpServitorController {
	base_url: String,
	http: Client,
}

#[derive(Debug, Error)]
#[cfg_attr(test, derive(Clone))]
pub enum ServitorError {
	#[error("invalid or disallowed unit name")]
	BadRequest,
	#[error("missing or invalid authentication token")]
	Unauthorized,
	#[error("The Servitor instance encountered an unexpected issue")]
	InternalServerError,
	#[error("Unexpected error: {status_code:?}, error: {error:?}")]
	Unexpected {
		status_code: Option<StatusCode>,
		error: Option<Arc<reqwest::Error>>,
	},
}

impl PartialEq for ServitorError {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::BadRequest, Self::BadRequest)
			| (Self::Unauthorized, Self::Unauthorized)
			| (Self::InternalServerError, Self::InternalServerError) => true,
			(
				Self::Unexpected {
					status_code: s1,
					error: e1,
				},
				Self::Unexpected {
					status_code: s2,
					error: e2,
				},
			) => {
				s1 == s2 && e1.as_ref().map(|e| e.to_string()) == e2.as_ref().map(|e| e.to_string())
			}
			_ => false,
		}
	}
}

impl From<reqwest::Error> for ServitorError {
	fn from(error: reqwest::Error) -> Self {
		ServitorError::Unexpected {
			status_code: None,
			error: Some(Arc::new(error)),
		}
	}
}

impl HttpServitorController {
	pub fn new(base_url: &str, token: Option<&SecretString>) -> anyhow::Result<Self> {
		static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

		let mut client = Client::builder()
			.user_agent(USER_AGENT)
			.timeout(Duration::from_secs(2));

		if let Some(token) = token {
			let mut auth_header =
				HeaderValue::from_str(format!("Bearer {}", token.expose_secret()).as_str())?;
			auth_header.set_sensitive(true);

			let headers = HeaderMap::from_iter([(header::AUTHORIZATION, auth_header)]);
			client = client.default_headers(headers);
		}

		Ok(Self {
			base_url: base_url.trim_end_matches('/').to_string(),
			http: client.build()?,
		})
	}

	fn check_response(response: Response) -> Result<Response, ServitorError> {
		match response.status() {
			StatusCode::OK => Ok(response),
			StatusCode::BAD_REQUEST => Err(ServitorError::BadRequest),
			StatusCode::UNAUTHORIZED => Err(ServitorError::Unauthorized),
			StatusCode::INTERNAL_SERVER_ERROR => Err(ServitorError::InternalServerError),
			_ => Err(ServitorError::Unexpected {
				status_code: Some(response.status()),
				error: response.error_for_status().map_err(Arc::new).err(),
			}),
		}
	}

	async fn post_request(&self, action: &str, unit_name: &str) -> Result<(), ServitorError> {
		let url = format!("{}/api/v1/services/{}/{}", self.base_url, unit_name, action);
		let response = self.http.post(&url).send().await?;
		Self::check_response(response)?;
		Ok(())
	}
}

impl ServitorHandler for HttpServitorController {
	async fn start(&self, unit_name: &str) -> Result<(), ServitorError> {
		self.post_request("start", unit_name).await
	}

	async fn stop(&self, unit_name: &str) -> Result<(), ServitorError> {
		self.post_request("stop", unit_name).await
	}

	async fn restart(&self, unit_name: &str) -> Result<(), ServitorError> {
		self.post_request("restart", unit_name).await
	}

	async fn reload(&self, unit_name: &str) -> Result<(), ServitorError> {
		self.post_request("reload", unit_name).await
	}

	async fn status(&self, unit_name: &str) -> Result<UnitStatus, ServitorError> {
		let url = format!("{}/api/v1/services/{}/status", self.base_url, unit_name);
		let response = self.http.get(&url).send().await?;
		let status = Self::check_response(response)?.json::<UnitStatus>().await?;
		Ok(status)
	}

	async fn health(&self) -> bool {
		let url = format!("{}/health", self.base_url);

		self.http
			.get(&url)
			.send()
			.await
			.map_or(false, |r| r.error_for_status().is_ok())
	}
}

#[cfg(test)]
pub mod tests {
	use super::*;
	use crate::data::BotData;
	use std::collections::BTreeMap;
	use std::sync::atomic::{AtomicUsize, Ordering};
	use std::sync::Arc;
	use chrono::TimeZone;
	use tokio::sync::Mutex;

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
		called_start: Arc<AtomicUsize>,
		called_stop: Arc<AtomicUsize>,
		called_restart: Arc<AtomicUsize>,
		called_reload: Arc<AtomicUsize>,
		called_status: Arc<AtomicUsize>,
		error: Arc<Mutex<Option<ServitorError>>>,
	}

	impl MockServitorController {
		pub fn new() -> Self {
			Self {
				called_start: Arc::new(AtomicUsize::new(0)),
				called_stop: Arc::new(AtomicUsize::new(0)),
				called_restart: Arc::new(AtomicUsize::new(0)),
				called_reload: Arc::new(AtomicUsize::new(0)),
				called_status: Arc::new(AtomicUsize::new(0)),
				error: Arc::new(Mutex::new(None)),
			}
		}

		pub fn assert_not_called(&self) {
			self.assert_called_times(0, 0, 0, 0, 0);
		}

		pub fn assert_called_times(
			&self,
			start: usize,
			stop: usize,
			restart: usize,
			reload: usize,
			status: usize,
		) {
			assert_eq!(
				self.called_start.load(Ordering::Relaxed),
				start,
				"Start called incorrect times"
			);
			assert_eq!(
				self.called_stop.load(Ordering::Relaxed),
				stop,
				"Stop called incorrect times"
			);
			assert_eq!(
				self.called_restart.load(Ordering::Relaxed),
				restart,
				"Restart called incorrect times"
			);
			assert_eq!(
				self.called_reload.load(Ordering::Relaxed),
				reload,
				"Reload called incorrect times"
			);
			assert_eq!(
				self.called_status.load(Ordering::Relaxed),
				status,
				"Status called incorrect times"
			);
		}

		pub async fn set_error(&self, error: ServitorError) {
			*self.error.lock().await = Some(error);
		}

		async fn check_for_error(&self) -> Result<(), ServitorError> {
			if let Some(err) = self.error.lock().await.clone() {
				Err(err)
			} else {
				Ok(())
			}
		}

		pub fn default_status(unit_name: &str) -> UnitStatus {
			UnitStatus {
				service: unit_name.to_string(),
				state: "active".to_string(),
				sub_state: "running".to_string(),
				since: Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap(),
			}
		}
	}

	impl ServitorHandler for MockServitorController {
		async fn start(&self, _unit_name: &str) -> Result<(), ServitorError> {
			self.called_start.fetch_add(1, Ordering::Relaxed);
			self.check_for_error().await
		}

		async fn stop(&self, _unit_name: &str) -> Result<(), ServitorError> {
			self.called_stop.fetch_add(1, Ordering::Relaxed);
			self.check_for_error().await
		}

		async fn restart(&self, _unit_name: &str) -> Result<(), ServitorError> {
			self.called_restart.fetch_add(1, Ordering::Relaxed);
			self.check_for_error().await
		}

		async fn reload(&self, _unit_name: &str) -> Result<(), ServitorError> {
			self.called_reload.fetch_add(1, Ordering::Relaxed);
			self.check_for_error().await
		}

		async fn status(&self, unit_name: &str) -> Result<UnitStatus, ServitorError> {
			self.called_status.fetch_add(1, Ordering::Relaxed);
			self.check_for_error().await?;

			Ok(Self::default_status(unit_name))
		}

		async fn health(&self) -> bool {
			true
		}
	}
}
