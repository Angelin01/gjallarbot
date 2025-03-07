use serenity::all::{RoleId, UserId};
use std::collections::BTreeSet;

pub trait AuthorizationInfo {
	fn authorized_users(&self) -> &BTreeSet<UserId>;
	fn authorized_roles(&self) -> &BTreeSet<RoleId>;
}
