INSERT INTO wgs 
 (id, url, name) VALUES
 (1, 'lamako', 'LAMAKO WGMBH');

INSERT INTO users
 (id, name, username, password_hash, revoke_before, wg) VALUES
 (1, 'Konni', 'kartonrad', '$pbkdf2-sha256$i=10000,l=32$R+UA953d9tO8VFqh8vF7TA$wmDkCKb9ScX6MfPJxKNf4dBM9MMHwEeVCsgzMvkqm48',
 'NOW', 1);

INSERT INTO users
 (id, name, username, password_hash, revoke_before, wg) VALUES
 (2, 'Laura', 'lamaura', '$pbkdf2-sha256$i=10000,l=32$R+UA953d9tO8VFqh8vF7TA$wmDkCKb9ScX6MfPJxKNf4dBM9MMHwEeVCsgzMvkqm48',
 'NOW', 1);

 INSERT INTO users
 (id, name, username, password_hash, revoke_before, wg) VALUES
 (3, 'Marie', 'marieee', '$pbkdf2-sha256$i=10000,l=32$R+UA953d9tO8VFqh8vF7TA$wmDkCKb9ScX6MfPJxKNf4dBM9MMHwEeVCsgzMvkqm48',
 'NOW', 1);
 

INSERT INTO costs (id, wg_id, name, amount, creditor_id, added_on) VALUES
(1, 1, 'Apotheke IBU/Tests/Masken', 15.0, 1, 'NOW');
INSERT INTO cost_shares (cost_id, debtor_id, paid) VALUES
(1, 1, true);
INSERT INTO cost_shares (cost_id, debtor_id, paid) VALUES
(1, 2, false);
INSERT INTO cost_shares (cost_id, debtor_id, paid) VALUES
(1, 3, false);

INSERT INTO costs (id, wg_id, name, amount, creditor_id, added_on) VALUES
(2, 1, 'Einkauf whatever', 22.0, 2, 'NOW');
INSERT INTO cost_shares (cost_id, debtor_id, paid) VALUES
(2, 1, false);
INSERT INTO cost_shares (cost_id, debtor_id, paid) VALUES
(2, 2, true);

INSERT INTO costs (id, wg_id, name, amount, creditor_id, added_on) VALUES
(3, 1, 'DM Kirschkernkissen', 9.90, 1, 'NOW');
INSERT INTO cost_shares (cost_id, debtor_id, paid) VALUES
(3, 1, true);
INSERT INTO cost_shares (cost_id, debtor_id, paid) VALUES
(3, 2, false);
INSERT INTO cost_shares (cost_id, debtor_id, paid) VALUES
(3, 3, false);