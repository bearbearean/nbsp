INSERT INTO nbsp_config (key, value)
VALUES
    ('nbsp_html_head_extra', NULL),
    ('nbsp_html_body_extra', NULL)
ON CONFLICT (key) DO NOTHING;
