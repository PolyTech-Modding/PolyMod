-- Add migration script here
CREATE TABLE owners (
    user_id BIGINT PRIMARY KEY,
    mod_name TEXT NOT NULL,
    checksums VARCHAR(64)[]
);
