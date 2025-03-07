use std::collections::BTreeMap;
use figment::providers::{Env, Format, Toml};
use figment::Figment;
use secrecy::SecretString;
use serde::Deserialize;

static ENV_PREFIX: &'static str = "GJ_";
static ENV_CONFIG_FILE: &'static str = "GJ_CONFIG_FILE";
static DEFAULT_CONFIG_FILE: &'static str = "gjallarbot.toml";

impl Config {
	pub(crate) fn load() -> figment::error::Result<Config> {
		let config_file = std::env::var(ENV_CONFIG_FILE).unwrap_or(DEFAULT_CONFIG_FILE.into());

		Figment::from(Toml::file(config_file))
			.merge(Env::prefixed(ENV_PREFIX).split("_").lowercase(false))
			.extract()
	}
}

#[derive(Deserialize)]
pub struct Config {
	pub bot: BotConfig,
	#[serde(default)]
	pub log: LogConfig,
	#[serde(default)]
	pub servitor: BTreeMap<String, ServitorConfig>,
}

#[derive(Deserialize)]
pub struct BotConfig {
	pub token: SecretString,
}

#[derive(Deserialize)]
pub struct LogConfig {
	pub filter: String,
}

impl Default for LogConfig {
	fn default() -> Self {
		LogConfig {
			filter: "gjallarbot=info".into(),
		}
	}
}

#[derive(Deserialize)]
pub struct ServitorConfig {
	pub url: String,
	#[serde(default)]
	pub token: Option<SecretString>,
}
