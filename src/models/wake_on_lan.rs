use diesel::prelude::*;
use chrono::{DateTime, Utc};

use crate::schema::*;
use crate::services::wake_on_lan::MacAddress;

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = wake_on_lan_machines)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct WakeOnLanMachine {
	pub id: i32,
	pub name: String,
	pub mac: MacAddress,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = wake_on_lan_machines)]
pub struct NewWakeOnLanMachine<'a> {
	pub name: &'a str,
	pub mac: &'a MacAddress,
}

#[derive(Debug, Queryable, Identifiable, Associations, Selectable)]
#[diesel(
    table_name = wake_on_lan_machines_authorized_users,
    primary_key(machine_id, user_id),
    belongs_to(WakeOnLanMachine, foreign_key = machine_id)
)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct WakeOnLanMachineAuthorizedUser {
	pub machine_id: i32,
	pub user_id: i64,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = wake_on_lan_machines_authorized_users)]
pub struct NewWakeOnLanMachineAuthorizedUser {
	pub machine_id: i32,
	pub user_id: i64,
}

#[derive(Debug, Queryable, Identifiable, Associations, Selectable)]
#[diesel(
    table_name = wake_on_lan_machines_authorized_roles,
    primary_key(machine_id, role_id),
    belongs_to(WakeOnLanMachine, foreign_key = machine_id)
)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct WakeOnLanMachineAuthorizedRole {
	pub machine_id: i32,
	pub role_id: i64,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = wake_on_lan_machines_authorized_roles)]
pub struct NewWakeOnLanMachineAuthorizedRole {
	pub machine_id: i32,
	pub role_id: i64,
}
