CREATE TABLE IF NOT EXISTS user_invite_settings (
    setting_id BIGINT NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id BIGINT NOT NULL UNIQUE REFERENCES users(user_id),
    available_invite_count BIGINT NOT NULL DEFAULT 0
);
