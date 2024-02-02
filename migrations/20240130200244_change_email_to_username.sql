ALTER TABLE users
    RENAME email TO username;
ALTER TABLE users
    ADD superuser boolean not null DEFAULT false,
    ADD permanent boolean not null DEFAULT false;
