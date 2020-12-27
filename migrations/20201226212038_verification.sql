-- Add migration script here
CREATE TABLE verification (
    id SERIAL NOT NULL,
    checksum VARCHAR(64) PRIMARY KEY,
    verifier_id BIGINT NOT NULL,
    is_good BOOLEAN NOT NULL,
    reason TEXT
);

ALTER TABLE tokens ADD COLUMN roles integer NOT NULL DEFAULT 0;
