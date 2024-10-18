mod persistent_data;
mod wake_on_lan;

use serde::{Deserialize, Serialize};
use wake_on_lan::WakeOnLanData;

pub use persistent_data::*;

#[derive(Deserialize, Serialize)]
pub struct Data {
    wake_on_lan: WakeOnLanData,
}
