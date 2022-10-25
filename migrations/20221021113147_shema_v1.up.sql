-- Add up migration script here
-- Add migration script here

CREATE TABLE uploads (
    id SERIAL PRIMARY KEY,
    extension VARCHAR(5) NOT NULL,
    original_filename VARCHAR(200),
    size_kb int NOT NULL,
    access_only_by_wg int,
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

alter table uploads
  ADD FOREIGN KEY (access_only_by_wg)
  references wgs (id);

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

-- COSTS
CREATE TABLE equal_balances (
    id SERIAL PRIMARY KEY,
    balanced_on timestamp with time zone NOT NULL,
    initiator_id int REFERENCES users(id) NOT NULL
);

CREATE TABLE costs (
    id SERIAL PRIMARY KEY,
    wg_id int REFERENCES wgs(id) NOT NULL,
    name VARCHAR(200) NOT NULL,
    amount NUMERIC(16,2) NOT NULL,
    creditor_id int REFERENCES users(id) NOT NULL,
    receit_id int REFERENCES uploads(id) UNIQUE,
    added_on timestamp with time zone NOT NULL,
    equal_balances int REFERENCES equal_balances(id)
    CHECK(amount > 0)
);

CREATE TABLE cost_shares (
    cost_id int REFERENCES costs(id) NOT NULL,
    debtor_id int REFERENCES users(id) NOT NULL,
    paid boolean NOT NULL,
    PRIMARY KEY(cost_id, debtor_id)
);