use serenity::all::{RoleId, UserId};

pub mod wake_on_lan;
pub mod servitor;

#[derive(Debug, PartialEq)]
pub enum DiscordEntity {
	User(UserId),
	Role(RoleId),
}
