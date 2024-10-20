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
