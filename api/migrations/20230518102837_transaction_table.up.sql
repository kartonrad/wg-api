-- Add up migration script here
CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    wg_id int REFERENCES wgs(id) NOT NULL,

	debtor_id int REFERENCES users(id) NOT NULL,
	creditor_id int REFERENCES users(id) NOT NULL,
	amount NUMERIC(16,2) NOT NULL,

    added_on timestamp with time zone NOT NULL,
    equal_balances int REFERENCES equal_balances(id)
    CHECK(amount > 0)
);