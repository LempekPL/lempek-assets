CREATE TABLE users
(
    id            UUID PRIMARY KEY     DEFAULT uuidv7(),
    login         TEXT UNIQUE NOT NULL,
    password_hash TEXT        NOT NULL,
    allow_upload  BOOLEAN     NOT NULL DEFAULT false
);

CREATE TABLE asset_space
(
    id       UUID PRIMARY KEY DEFAULT uuidv7(),
    owner_id UUID REFERENCES users (id) ON DELETE CASCADE,
    name     TEXT NOT NULL
);