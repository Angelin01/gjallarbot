-- Wake on Lan Machines
DROP TABLE IF EXISTS wake_on_lan_machines_authorized_roles;
DROP TABLE IF EXISTS wake_on_lan_machines_authorized_users;

DROP TRIGGER IF EXISTS wake_on_lan_machines_update_ts;

DROP INDEX IF EXISTS idx_wol_machines_name;

DROP TABLE IF EXISTS wake_on_lan_machines;

-- Servitor
DROP TABLE IF EXISTS servitor_server_authorized_roles;
DROP TABLE IF EXISTS servitor_server_authorized_users;

DROP TRIGGER IF EXISTS servitor_servers_update_ts;

DROP INDEX IF EXISTS idx_servitor_server_name;

DROP TABLE IF EXISTS servitor_servers;
