use crate::data::authorization::AuthorizationInfo;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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

impl AuthorizationInfo for ServerInfo {
	fn authorized_users(&self) -> &BTreeSet<serenity::UserId> {
		&self.authorized_users
	}
	fn authorized_roles(&self) -> &BTreeSet<serenity::RoleId> {
		&self.authorized_roles
	}
}
