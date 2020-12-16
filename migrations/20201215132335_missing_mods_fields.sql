-- Add migration script here
CREATE TYPE VERIFICATION_LEVEL AS ENUM (
    'Unsafe',
    'AutoVerified',
    'ManualVerified',
    'PolyTechCore'
);

ALTER TABLE mods ADD COLUMN verification VERIFICATION_LEVEL;
ALTER TABLE mods ADD COLUMN dowloads bigint NOT NULL DEFAULT 0;
ALTER TABLE mods ADD COLUMN uploaded TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP;
