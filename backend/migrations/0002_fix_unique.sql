ALTER TABLE folders
    DROP CONSTRAINT IF EXISTS unique_parent_name,
    ADD CONSTRAINT unique_parent_name
        UNIQUE NULLS NOT DISTINCT (parent_id, name);