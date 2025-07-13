CREATE
    OR REPLACE FUNCTION uuidv7(
)
    RETURNS uuid
    LANGUAGE 'sql'
    COST 100
    VOLATILE PARALLEL UNSAFE
AS
$BODY$
select encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing
                                      substring(int8send((extract(epoch from clock_timestamp()) * 1000)::bigint) from
                                                3)
                                      from 1 for 6),
                              52, 1),
                      53, 1), 'hex') ::uuid;
$BODY$;

CREATE TABLE users
(
    id         UUID PRIMARY KEY     DEFAULT uuidv7(),
    login      TEXT UNIQUE NOT NULL,
    password   TEXT        NOT NULL,
    admin      BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE folders
(
    id         UUID PRIMARY KEY                               DEFAULT uuidv7(),
    parent_id  UUID REFERENCES folders (id) ON DELETE CASCADE DEFAULT NULL,
    owner_id   UUID        NOT NULL,
    name       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL                           DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL                           DEFAULT NOW(),
    UNIQUE (parent_id, name)
);

CREATE TABLE files
(
    id         UUID PRIMARY KEY     DEFAULT uuidv7(),
    folder_id  UUID                 DEFAULT NULL REFERENCES folders (id) ON DELETE CASCADE,
    owner_id   UUID        NOT NULL,
    name       TEXT        NOT NULL,
    size       BIGINT      NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE permissions
(
    id        UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id   UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    folder_id UUID NOT NULL REFERENCES folders (id) ON DELETE CASCADE,
    read      BOOLEAN          DEFAULT FALSE,
    modify    BOOLEAN          DEFAULT FALSE,
    edit      BOOLEAN          DEFAULT FALSE,
    UNIQUE (user_id, folder_id)
);

CREATE UNIQUE INDEX idx_permissions_user_folder ON permissions (user_id, folder_id);

CREATE INDEX idx_folders_parent_id ON folders (parent_id);

CREATE INDEX idx_files_folder_id ON files (folder_id);

CREATE OR REPLACE FUNCTION set_updated_at()
    RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_users_updated
    BEFORE UPDATE
    ON users
    FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_folders_updated
    BEFORE UPDATE
    ON folders
    FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
CREATE TRIGGER set_files_updated
    BEFORE UPDATE
    ON files
    FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

