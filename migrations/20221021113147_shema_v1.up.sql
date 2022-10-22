-- Add up migration script here
-- Add migration script here

CREATE TABLE uploads (
    id SERIAL PRIMARY KEY,
    extension VARCHAR(5) NOT NULL,
    original_filename VARCHAR(200),
    size_kb int NOT NULL,
    CHECK( size_kb > 0 )
);


CREATE TABLE wgs (
    id SERIAL PRIMARY KEY,
    url VARCHAR(40) UNIQUE NOT NULL,
    CHECK ( url SIMILAR TO '[abcdefghijklmnopqrstuvwxyz0123456789\-_]+'),

    name VARCHAR(200) NOT NULL DEFAULT '<Wg-Name>',
    description TEXT NOT NULL DEFAULT '<Wg-Tagline>',

    profile_pic int REFERENCES uploads(id) UNIQUE,
    header_pic int REFERENCES uploads(id) UNIQUE
);


CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    profile_pic int REFERENCES uploads(id) UNIQUE,
    name VARCHAR(200) NOT NULL DEFAULT '<Unbenannter Mensch>',
    bio TEXT NOT NULL DEFAULT '<Beschreibung>',

    username VARCHAR(40) UNIQUE NOT NULL,
    CHECK ( username SIMILAR TO '[abcdefghijklmnopqrstuvwxyz0123456789\-_]+'),
    password_hash CHAR(100) NOT NULL,
    revoke_before timestamp NOT NULL,

    wg int REFERENCES wgs(id)
);
