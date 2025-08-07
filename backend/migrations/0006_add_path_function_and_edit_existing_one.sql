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
    RETURNS TABLE(uuid uuid, path text)
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