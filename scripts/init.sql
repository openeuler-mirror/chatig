CREATE TABLE IF NOT EXISTS file_object (
    id SERIAL PRIMARY KEY,
    object TEXT NOT NULL,
    bytes INTEGER NOT NULL,
    created_at BIGINT NOT NULL,
    filename TEXT NOT NULL,
    purpose TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS invitation_code (
    id SERIAL PRIMARY KEY,
    users TEXT NOT NULL,
    origination TEXT,
    telephone TEXT,
    email TEXT,
    created_at BIGINT NOT NULL,
    code TEXT NOT NULL,
    UNIQUE (code)
);

CREATE TABLE IF NOT EXISTS project_object (
    id TEXT PRIMARY KEY,
    object TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    archived_at BIGINT,
    status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_object (
    id TEXT PRIMARY KEY,
    object TEXT NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    role TEXT NOT NULL,
    added_at BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS models (
    id TEXT PRIMARY KEY,
    object TEXT NOT NULL,
    created BIGINT NOT NULL,
    owned_by TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS services (
    id TEXT PRIMARY KEY,
    servicetype TEXT NOT NULL,
    status TEXT NOT NULL,
    url TEXT NOT NULL,
    model_name TEXT NOT NULL,
    active_model TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS models_service (
    serviceid TEXT NOT NULL,
    modelid TEXT NOT NULL,
    PRIMARY KEY (serviceid, modelid),
    FOREIGN KEY (serviceid) REFERENCES services(id) ON DELETE CASCADE
);