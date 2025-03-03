CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    login TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL
);

CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    owner_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL
);
