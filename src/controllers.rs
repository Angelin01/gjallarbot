use serenity::all::{RoleId, UserId};

pub mod wake_on_lan;
pub mod servitor;

#[derive(Debug, PartialEq)]
pub enum DiscordEntity {
	User(UserId),
	Role(RoleId),
}

#[cfg(test)]
pub mod tests {
	use serenity::all::{Member, RoleId, User, UserId};

	pub fn mock_author_dms(id: UserId) -> User {
		let mut user = User::default();
		user.id = id;
		user.name = "mock_author".to_string();
		user
	}

	pub fn mock_author_guild(id: UserId, roles: Vec<RoleId>) -> User {
		let mut user = mock_author_dms(id);

		let mut member = Member::default();
		member.roles = roles;

		user.member = Some(Box::new(member.into()));

		user
	}
}
