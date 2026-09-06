INSERT INTO nbsp_config (key)
VALUES ('nbsp_cookies_key'), ('nbsp_jwt_signing_key')
ON CONFLICT (key) DO NOTHING;
