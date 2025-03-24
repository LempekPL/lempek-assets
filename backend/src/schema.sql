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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    path       TEXT        NOT NULL
);

CREATE TABLE files
(
    id         UUID PRIMARY KEY     DEFAULT uuidv7(),
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    folder_id  UUID                 DEFAULT NULL REFERENCES folders (id) ON DELETE CASCADE,
    name       TEXT        NOT NULL,
    path       TEXT        NOT NULL UNIQUE,
    size       BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
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

CREATE OR REPLACE FUNCTION public.get_folder_root(
    folder_id uuid)
    RETURNS uuid
    LANGUAGE 'sql'
    COST 100
    VOLATILE PARALLEL UNSAFE
AS
$BODY$
WITH RECURSIVE folder_tree AS (SELECT id, parent_id
                               FROM folders
                               WHERE id = folder_id

                               UNION ALL

                               SELECT f.id, f.parent_id
                               FROM folders f
                                        JOIN folder_tree ft ON f.id = ft.parent_id)
SELECT id
FROM folder_tree
WHERE parent_id IS NULL
LIMIT 1;
$BODY$;

CREATE OR REPLACE FUNCTION public.set_folder_path()
    RETURNS trigger
    LANGUAGE 'plpgsql'
    COST 100
    VOLATILE NOT LEAKPROOF
AS
$BODY$
BEGIN
    IF NEW.parent_id IS NULL THEN
        NEW.path := NEW.name;
    ELSE
        NEW.path := get_folder_path(NEW.parent_id) || '/' || NEW.name;
    END IF;
    RETURN NEW;
END;
$BODY$;

CREATE OR REPLACE TRIGGER folders_set_path_trigger
    BEFORE INSERT OR UPDATE OF name, parent_id
    ON public.folders
    FOR EACH ROW
EXECUTE FUNCTION public.set_folder_path();