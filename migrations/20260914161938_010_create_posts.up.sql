CREATE TABLE IF NOT EXISTS posts (
    post_id BIGINT NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    creator_user_id BIGINT NOT NULL REFERENCES users(user_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT (now()),
    title TEXT NOT NULL,
    markdown_source TEXT NOT NULL,
    html_rendered TEXT NOT NULL
);
