CREATE TABLE users
(
    id         UUID PRIMARY KEY     DEFAULT uuidv7(),
    login      TEXT UNIQUE NOT NULL,
    username   TEXT UNIQUE NOT NULL,
    password   TEXT        NOT NULL,
    admin      BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE folders
(
    id         UUID PRIMARY KEY                               DEFAULT uuidv7(),
    parent_id  UUID REFERENCES folders (id) ON DELETE CASCADE DEFAULT NULL,
    owner_id   UUID        NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    name       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL                           DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL                           DEFAULT NOW(),
    UNIQUE NULLS NOT DISTINCT (parent_id, name)
);

CREATE TABLE files
(
    id         UUID PRIMARY KEY                               DEFAULT uuidv7(),
    folder_id  UUID REFERENCES folders (id) ON DELETE CASCADE DEFAULT NULL,
    owner_id   UUID        NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    name       TEXT        NOT NULL,
    size       BIGINT      NOT NULL                           DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL                           DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL                           DEFAULT NOW()
);

CREATE TABLE permissions
(
    id        UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id   UUID    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    folder_id UUID REFERENCES folders (id) ON DELETE CASCADE,
    read      BOOLEAN NOT NULL DEFAULT FALSE,
    modify    BOOLEAN NOT NULL DEFAULT FALSE,
    edit      BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE (user_id, folder_id)
);

CREATE TABLE user_tokens
(
    id            UUID PRIMARY KEY     DEFAULT uuidv7(),
    user_id       UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE,
    refresh_token UUID        NOT NULL DEFAULT gen_random_uuid(),
    user_agent    TEXT,
    country       TEXT,
    region        TEXT,
    city          TEXT,
    expires_at    TIMESTAMPTZ NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, refresh_token)
);


CREATE UNIQUE INDEX idx_permissions_user_folder ON permissions (user_id, folder_id);

CREATE INDEX idx_folders_parent_id ON folders (parent_id);

CREATE INDEX idx_files_folder_id ON files (folder_id);

CREATE INDEX idx_refresh_tokens_user_id ON user_tokens (user_id);
CREATE UNIQUE INDEX idx_refresh_tokens_token ON user_tokens (user_id, refresh_token);

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

CREATE OR REPLACE FUNCTION get_folder_path(start_id UUID)
    RETURNS TEXT
    LANGUAGE SQL
AS
$$
WITH RECURSIVE whole_path AS (SELECT id, parent_id, name::text AS path
                              FROM folders
                              WHERE id = start_id

                              UNION ALL

                              SELECT f.id, f.parent_id, f.name || '/' || p.path
                              FROM folders f
                                       JOIN whole_path p ON f.id = p.parent_id)
SELECT path
FROM whole_path
WHERE parent_id IS NULL;
$$;

CREATE OR REPLACE FUNCTION get_folder_path(start_id UUID)
    RETURNS TEXT
    LANGUAGE SQL
AS
$$
WITH RECURSIVE whole_path AS (SELECT id, parent_id, name::text AS path
                              FROM folders
                              WHERE id IS NOT DISTINCT FROM start_id

                              UNION ALL

                              SELECT f.id, f.parent_id, f.name || '/' || p.path
                              FROM folders f
                                       JOIN whole_path p ON f.id = p.parent_id)
SELECT path
FROM whole_path
WHERE parent_id IS NULL;
$$;

CREATE OR REPLACE FUNCTION get_folder_uuid_path(start_id UUID)
    RETURNS TABLE
            (
                id   uuid,
                name text
            )
    LANGUAGE SQL
AS
$$
WITH
    RECURSIVE ancestors
                  AS
                  (SELECT id,
                          parent_id,
                          name,
                          1 AS lvl
                   FROM folders
                   WHERE id IS NOT DISTINCT FROM start_id

                   UNION ALL

                   SELECT f.id,
                          f.parent_id,
                          f.name,
                          a.lvl + 1
                   FROM folders f
                            JOIN ancestors a ON a.parent_id = f.id)
SELECT id, name
FROM ancestors
ORDER BY lvl DESC;
$$;