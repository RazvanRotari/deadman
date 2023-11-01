-- Add up migration script here
CREATE TABLE user (
    user_id TEXT NOT NULL,
    last_call INTEGER NOT NULL,
    interval_minutes INTEGER NOT NULL
);

INSERT INTO user (user_id, last_call, interval_minutes)
VALUES (
    'test_id',
    203252252,
    30
)

