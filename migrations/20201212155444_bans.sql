-- Add migration script here
ALTER TABLE tokens ADD COLUMN is_banned boolean NOT NULL DEFAULT false;
