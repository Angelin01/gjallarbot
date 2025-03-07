use crate::data::authorization::AuthorizationInfo;
use crate::services::wake_on_lan::MacAddress;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub type WakeOnLanData = BTreeMap<String, WakeOnLanMachineInfo>;

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct WakeOnLanMachineInfo {
	pub mac: MacAddress,
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	#[serde(default)]
	pub authorized_users: BTreeSet<serenity::UserId>,
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	#[serde(default)]
	pub authorized_roles: BTreeSet<serenity::RoleId>,
}

impl AuthorizationInfo for WakeOnLanMachineInfo {
	fn authorized_users(&self) -> &BTreeSet<serenity::UserId> {
		&self.authorized_users
	}
	fn authorized_roles(&self) -> &BTreeSet<serenity::RoleId> {
		&self.authorized_roles
	}
}
