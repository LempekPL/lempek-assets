BEGIN;

ALTER TABLE public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);

COMMIT;