CREATE EXTENSION "uuid-ossp";

CREATE TABLE invites (
    invite_id BIGINT NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    invite_code UUID NOT NULL UNIQUE DEFAULT (uuid_generate_v4()),
    user_creator_id BIGINT NOT NULL REFERENCES users(user_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT (now()),
    user_consumer_id BIGINT REFERENCES users(user_id),
    consumed_at TIMESTAMPTZ
);

/* Create an invite code using the system nbsp user, so that the first real user can register */
INSERT INTO invites (user_creator_id) VALUES (-1);
