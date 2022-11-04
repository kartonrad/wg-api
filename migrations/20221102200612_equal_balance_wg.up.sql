-- Add up migration script here
ALTER TABLE equal_balances ADD COLUMN wg_id int REFERENCES wgs(id) NOT NULL;