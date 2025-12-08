// @generated automatically by Diesel CLI.

diesel::table! {
	servitor_server_authorized_roles (server_id, role_id) {
		server_id -> Integer,
		role_id -> Integer,
		created_at -> Text,
	}
}

diesel::table! {
	servitor_server_authorized_users (server_id, user_id) {
		server_id -> Integer,
		user_id -> Integer,
		created_at -> Text,
	}
}

diesel::table! {
	servitor_servers (id) {
		id -> Integer,
		name -> Text,
		servitor -> Text,
		unit_name -> Text,
		created_at -> Text,
		updated_at -> Text,
	}
}

diesel::table! {
	wake_on_lan_machines (id) {
		id -> Integer,
		name -> Text,
		mac -> Text,
		created_at -> Text,
		updated_at -> Text,
	}
}

diesel::table! {
	wake_on_lan_machines_authorized_roles (machine_id, role_id) {
		machine_id -> Integer,
		role_id -> Integer,
		created_at -> Text,
	}
}

diesel::table! {
	wake_on_lan_machines_authorized_users (machine_id, user_id) {
		machine_id -> Integer,
		user_id -> Integer,
		created_at -> Text,
	}
}

diesel::joinable!(servitor_server_authorized_roles -> servitor_servers (server_id));
diesel::joinable!(servitor_server_authorized_users -> servitor_servers (server_id));
diesel::joinable!(wake_on_lan_machines_authorized_roles -> wake_on_lan_machines (machine_id));
diesel::joinable!(wake_on_lan_machines_authorized_users -> wake_on_lan_machines (machine_id));

diesel::allow_tables_to_appear_in_same_query!(
	servitor_server_authorized_roles,
	servitor_server_authorized_users,
	servitor_servers,
	wake_on_lan_machines,
	wake_on_lan_machines_authorized_roles,
	wake_on_lan_machines_authorized_users,
);
