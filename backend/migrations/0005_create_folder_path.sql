CREATE OR REPLACE FUNCTION get_folder_path(start_id UUID)
    RETURNS TEXT
    LANGUAGE SQL
AS $$
    WITH RECURSIVE whole_path AS (
    SELECT id, parent_id, name::text AS path
    FROM folders
    WHERE id = start_id

    UNION ALL

    SELECT f.id, f.parent_id, f.name || '/' || p.path
    FROM folders f
             JOIN whole_path p ON f.id = p.parent_id
)
SELECT path FROM whole_path WHERE parent_id IS NULL;
$$;