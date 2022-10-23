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
 (2, 'Marie', 'marieee', '$pbkdf2-sha256$i=10000,l=32$R+UA953d9tO8VFqh8vF7TA$wmDkCKb9ScX6MfPJxKNf4dBM9MMHwEeVCsgzMvkqm48',
 'NOW', 1);
 


