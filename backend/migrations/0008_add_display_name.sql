BEGIN;

ALTER TABLE public.users
    ADD COLUMN username text COLLATE pg_catalog."default";

UPDATE public.users
SET username = login
WHERE username IS NULL;

ALTER TABLE public.users
    ALTER COLUMN username SET NOT NULL;

COMMIT;