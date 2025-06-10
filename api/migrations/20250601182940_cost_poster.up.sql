-- Add up migration script here

ALTER TABLE costs ADD COLUMN poster_id int REFERENCES users(id);
UPDATE costs SET poster_id = creditor_id;
ALTER TABLE costs ALTER COLUMN poster_id SET NOT NULL;
