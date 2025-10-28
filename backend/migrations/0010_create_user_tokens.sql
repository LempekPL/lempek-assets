CREATE TABLE user_tokens
(
    id            UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id       UUID REFERENCES users (id),
    refresh_token UUID      NOT NULL DEFAULT gen_random_uuid(),
    expires_at    TIMESTAMP NOT NULL,
    created_at    TIMESTAMP        DEFAULT NOW()
);

CREATE INDEX idx_refresh_tokens_user_id ON user_tokens (user_id);
CREATE INDEX idx_refresh_tokens_token ON user_tokens (refresh_token);