-- Add up migration script here
CREATE TABLE user (
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    telegram_id INTEGER NOT NULL,
    last_call INTEGER NOT NULL,
    interval_minutes INTEGER NOT NULL
);
