CREATE TABLE IF NOT EXISTS users (
    user_id BIGINT NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    username TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT (now()),
    password_hash TEXT
);

/* Create a unique index on the lowercased username, to preserve the casing of the actual username
in the column but still have the unique constraint and an index. */
CREATE UNIQUE INDEX IF NOT EXISTS users_username_key ON users (lower(username));

/* Create a system user called nbsp with the special -1 user_id */
INSERT INTO users (user_id, username)
OVERRIDING SYSTEM VALUE
VALUES (-1, 'nbsp')
ON CONFLICT (user_id) DO NOTHING;
