-- Add migration script here
ALTER TABLE team_members ADD CONSTRAINT double_pk PRIMARY KEY (member, team_id);
