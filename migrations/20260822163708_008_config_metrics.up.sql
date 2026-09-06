INSERT INTO nbsp_config (key, value)
VALUES ('nbsp_enable_prometheus_metrics', 'false')
ON CONFLICT (key) DO NOTHING;
