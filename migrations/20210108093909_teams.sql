-- Add migration script here
CREATE TABLE teams (
    id SERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL
);

CREATE TABLE team_members (
    team_id INTEGER NOT NULL,
    member BIGINT NOT NULL,
    roles INTEGER NOT NULL DEFAULT 0,

    CONSTRAINT fk_team_id
        FOREIGN KEY(team_id)
        REFERENCES teams(id), -- relation "id" does not exist

    CONSTRAINT fk_member
        FOREIGN KEY(member)
        REFERENCES tokens(user_id)
);

ALTER TABLE tokens ADD COLUMN is_team boolean NOT NULL DEFAULT false;
ALTER TABLE owners ADD COLUMN is_team boolean NOT NULL DEFAULT false;
ALTER TABLE tokens RENAME COLUMN user_id TO owner_id;
ALTER TABLE owners RENAME COLUMN user_id TO owner_id;
