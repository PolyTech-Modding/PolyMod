-- Add migration script here
CREATE TYPE CATEGORIES AS ENUM (
    'API',
    'Editor',
    'Cheat',
    'Models',
    'Utilities',
    'Physics',
    'Fun',
    'Cosmetic'
);

CREATE TABLE mods(
    id SERIAL UNIQUE,
    checksum VARCHAR(64) NOT NULL UNIQUE, -- mod package checksum

    name TEXT NOT NULL, -- mod name
    version TEXT NOT NULL, -- mod version SemVer
    description TEXT NOT NULL, -- Short description

    -- Double NOT NULL, but idk how
    repository_git TEXT,
    repository_hg TEXT,

    authors TEXT[],
    documentation TEXT, -- URL
    readme TEXT, -- the readme contents
    homepage TEXT, -- URL
    license TEXT, -- OSI Licence or text for license-file
    keywords TEXT[],
    categories CATEGORIES[],
    build_script TEXT, -- Build shell script

    native_lib_checksums TEXT[], -- Checksums of native library files.
    dependencies_checksums VARCHAR(64)[], -- Checksums of dependency files.

    metadata TEXT[], -- Extra metadata

    UNIQUE(name, version),
    PRIMARY KEY(id, checksum)
);

CREATE FUNCTION check_double_nullable() RETURNS TRIGGER AS $nullable_repository$
BEGIN
    IF
        NEW.repository_git IS NULL
        AND
        NEW.repository_hg IS NULL
        THEN
        RAISE EXCEPTION 'Must specify a repository';
    END IF;

    RETURN NEW;
END;
$nullable_repository$ LANGUAGE plpgsql;

CREATE TRIGGER nullable_repository
    BEFORE INSERT OR UPDATE
    ON mods
    FOR EACH ROW
    EXECUTE FUNCTION
    check_double_nullable();
