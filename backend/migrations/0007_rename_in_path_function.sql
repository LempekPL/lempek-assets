DROP FUNCTION IF EXISTS get_folder_uuid_path(uuid);

CREATE OR REPLACE FUNCTION get_folder_uuid_path(start_id UUID)
    RETURNS TABLE(id uuid, name text)
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