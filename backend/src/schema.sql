CREATE TABLE users
(
    id            UUID PRIMARY KEY     DEFAULT uuidv7(),
    login         TEXT UNIQUE NOT NULL,
    password_hash TEXT        NOT NULL,
    allow_upload  BOOLEAN     NOT NULL DEFAULT FALSE
);

CREATE TABLE folders
(
    id         UUID PRIMARY KEY     DEFAULT uuidv7(),
    name       TEXT        NOT NULL,
    parent_id  UUID REFERENCES folders (id) ON DELETE CASCADE,
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE files
(
    id          UUID PRIMARY KEY     DEFAULT uuidv7(),
    user_id     UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    folder_id   UUID REFERENCES folders (id) ON DELETE CASCADE,
    filename    TEXT        NOT NULL,
    filepath    TEXT        NOT NULL UNIQUE,
    size        BIGINT,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION uuidv7(
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
                      53, 1), 'hex')::uuid;
$BODY$;

CREATE OR REPLACE FUNCTION get_folder_path(
    folder_id uuid)
    RETURNS text
    LANGUAGE 'sql'
    COST 100
    VOLATILE PARALLEL UNSAFE
AS
$BODY$
WITH
    RECURSIVE folder_hierarchy
                  AS
                  (SELECT id,
                          name,
                          parent_id,
                          name AS path
                   FROM folders
                   WHERE id = folder_id
                   UNION ALL
                   SELECT f.id,
                          f.name,
                          f.parent_id,
                          f.name || '/' || fh.path AS path
                   FROM folders f
                            INNER JOIN folder_hierarchy fh ON f.id = fh.parent_id)
SELECT path
FROM folder_hierarchy
ORDER BY LENGTH(path) DESC
LIMIT 1;
$BODY$;