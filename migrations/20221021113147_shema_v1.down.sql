-- Add down migration script here
DROP TABLE users;
alter table uploads
  DROP FOREIGN KEY (access_only_by_wg);
DROP TABLE wgs;
DROP TABLE uploads;