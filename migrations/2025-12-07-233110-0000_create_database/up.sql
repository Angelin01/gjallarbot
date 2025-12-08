-- Wake on Lan Machines
CREATE TABLE wake_on_lan_machines(
     id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
     name TEXT UNIQUE NOT NULL,
     mac TEXT NOT NULL,
     created_at TEXT NOT NULL,
     updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_wol_machines_name ON wake_on_lan_machines(name);

CREATE TABLE wake_on_lan_machines_authorized_users (
    machine_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (machine_id, user_id),
    FOREIGN KEY(machine_id) REFERENCES wake_on_lan_machines(id) ON DELETE CASCADE
);

CREATE TABLE wake_on_lan_machines_authorized_roles (
    machine_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (machine_id, role_id),
    FOREIGN KEY(machine_id) REFERENCES wake_on_lan_machines(id) ON DELETE CASCADE
);

-- Servitor
CREATE TABLE servitor_servers(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    servitor TEXT NOT NULL,
    unit_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_servitor_server_name ON servitor_servers(name);

CREATE TABLE servitor_server_authorized_users (
    server_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (server_id, user_id),
    FOREIGN KEY(server_id) REFERENCES servitor_servers(id) ON DELETE CASCADE
);

CREATE TABLE servitor_server_authorized_roles (
    server_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (server_id, role_id),
    FOREIGN KEY(server_id) REFERENCES servitor_servers(id) ON DELETE CASCADE
);
