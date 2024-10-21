use poise::serenity_prelude as serenity;
use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use crate::services::wake_on_lan::MacAddress;

pub type WakeOnLanData = BTreeMap<String, WakeOnLanMachineInfo>;

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct WakeOnLanMachineInfo {
    pub mac: MacAddress,
    pub authorized_users: BTreeSet<serenity::UserId>,
    pub authorized_roles: BTreeSet<serenity::RoleId>,
}
