-- Add up migration script here

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