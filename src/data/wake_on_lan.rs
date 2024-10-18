use poise::serenity_prelude as serenity;
use std::collections::{HashMap, HashSet};
use crate::services::wake_on_lan::MacAddress;

pub type WakeOnLanData = HashMap<String, WakeOnLanMachineInfo>;

pub struct WakeOnLanMachineInfo {
    mac: MacAddress,
    authorized_users: HashSet<serenity::UserId>,
    authorized_roles: HashSet<serenity::RoleId>,
}
