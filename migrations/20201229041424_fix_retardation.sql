-- Add migration script here
ALTER TABLE owners DROP CONSTRAINT owners_pkey;
ALTER TABLE owners ADD CONSTRAINT owners_pkey PRIMARY KEY (mod_name);
