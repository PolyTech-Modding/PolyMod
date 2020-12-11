-- Add migration script here
CREATE TABLE tokens(
    id SERIAL NOT NULL,
    user_id bigint PRIMARY KEY NOT NULL,
    token text NOT NULL,
    email text NOT NULL
);
