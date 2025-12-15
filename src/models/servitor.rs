use diesel::prelude::*;
use chrono::{DateTime, Utc};

use crate::schema::*;

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = servitor_servers)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ServitorServer {
	pub id: i32,
	pub name: String,
	pub servitor: String,
	pub unit_name: String,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = servitor_servers)]
pub struct NewServitorServer<'a> {
	pub name: &'a str,
	pub servitor: &'a str,
	pub unit_name: &'a str,
}

#[derive(Debug, Queryable, Identifiable, Associations, Selectable)]
#[diesel(
    table_name = servitor_server_authorized_users,
    primary_key(server_id, user_id),
    belongs_to(ServitorServer, foreign_key = server_id)
)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ServitorServerAuthorizedUser {
	pub server_id: i32,
	pub user_id: i64,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = servitor_server_authorized_users)]
pub struct NewServitorServerAuthorizedUser {
	pub server_id: i32,
	pub user_id: i64,
}

#[derive(Debug, Queryable, Identifiable, Associations, Selectable)]
#[diesel(
    table_name = servitor_server_authorized_roles,
    primary_key(server_id, role_id),
    belongs_to(ServitorServer, foreign_key = server_id)
)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ServitorServerAuthorizedRole {
	pub server_id: i32,
	pub role_id: i64,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = servitor_server_authorized_roles)]
pub struct NewServitorServerAuthorizedRole {
	pub server_id: i32,
	pub role_id: i64,
}
