-- Add up migration script here
ALTER TABLE user
ADD COLUMN last_notification INTEGER;
