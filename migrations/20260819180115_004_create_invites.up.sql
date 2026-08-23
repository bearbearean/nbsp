CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS invites (
    invite_id BIGINT NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    invite_code UUID NOT NULL UNIQUE DEFAULT (uuid_generate_v4()),
    user_creator_id BIGINT NOT NULL REFERENCES users(user_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT (now()),
    user_consumer_id BIGINT REFERENCES users(user_id),
    consumed_at TIMESTAMPTZ
);

/* Create an invite code using the system nbsp user, so that the first real user can register
Don't create any invites if there are already records in the table */
INSERT INTO invites (user_creator_id)
SELECT user_creator_id FROM (SELECT -1 "user_creator_id")
WHERE (SELECT COUNT(*) FROM invites) <= 0;
