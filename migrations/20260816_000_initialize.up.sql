CREATE TABLE IF NOT EXISTS nbsp_config (
    config_id BIGINT NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    value TEXT
);

INSERT INTO nbsp_config (key, value)
VALUES
    ('nbsp_base_url', 'https://nbsp.example.com'),
    ('nbsp_homepage_notice', NULL)
ON CONFLICT (key) DO NOTHING;
