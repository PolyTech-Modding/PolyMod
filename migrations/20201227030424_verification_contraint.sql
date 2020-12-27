-- Add migration script here
ALTER TABLE verification DROP CONSTRAINT verification_pkey;
ALTER TABLE verification ADD CONSTRAINT verification_pkey PRIMARY KEY (checksum, verifier_id);
