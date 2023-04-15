-- Add down migration script here
DROP TABLE equal_balances;
DROP TABLE costs;
DROP TABLE cost_shares;

DROP TABLE users;
alter table uploads
  DROP FOREIGN KEY (access_only_by_wg);
DROP TABLE wgs;
DROP TABLE uploads;