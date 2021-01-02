-- Add migration script here
ALTER TABLE mods ADD COLUMN license_filename TEXT;
ALTER TABLE mods ADD COLUMN readme_filename TEXT;
