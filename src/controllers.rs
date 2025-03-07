use serenity::all::{Member, RoleId, User, UserId};
use crate::data::authorization::AuthorizationInfo;

pub mod wake_on_lan;
pub mod servitor;

#[derive(Debug, PartialEq)]
pub enum DiscordEntity {
	User(UserId),
	Role(RoleId),
}

fn is_user_authorized<T: AuthorizationInfo>(
	author: &User,
	member: Option<&Member>,
	info: &T,
) -> bool {
	info.authorized_users().contains(&author.id)
		|| member.map_or(false, |m| {
		m.roles.iter().any(|&role| info.authorized_roles().contains(&role))
	})
}


#[cfg(test)]
pub mod tests {
	use serenity::all::{Member, RoleId, User, UserId};

	pub fn mock_author_dms(id: UserId) -> (User, Option<Member>) {
		let mut user = User::default();
		user.id = id;
		user.name = "mock_author".to_string();

		(user, None)
	}

	pub fn mock_author_guild(id: UserId, roles: Vec<RoleId>) -> (User, Option<Member>) {
		let (user, _) = mock_author_dms(id); // Reuse base user

		let mut member = Member::default();
		member.roles = roles;
		member.user = user.clone();

		(user, Some(member))
	}
}
