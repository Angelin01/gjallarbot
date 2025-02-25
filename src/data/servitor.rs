use poise::serenity_prelude as serenity;
use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};

pub type ServitorData = BTreeMap<String, ServerInfo>;

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct ServerInfo {
	pub servitor: String,
	pub unit_name: String,
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	#[serde(default)]
	pub authorized_users: BTreeSet<serenity::UserId>,
	#[serde(skip_serializing_if = "BTreeSet::is_empty")]
	#[serde(default)]
	pub authorized_roles: BTreeSet<serenity::RoleId>,
}
