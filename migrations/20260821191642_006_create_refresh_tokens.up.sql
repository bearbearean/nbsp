CREATE TABLE refresh_tokens (
    token_id BIGINT NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(user_id),
    refresh_token UUID UNIQUE NOT NULL DEFAULT (uuid_generate_v4()),
    created_at TIMESTAMPTZ NOT NULL DEFAULT (now()),
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT (now())
);
