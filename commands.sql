drop table if exists users;
create table users (
    id serial primary key,
    email text not null,
    username text not null,
    password text not null,
    UNIQUE (email),
    UNIQUE (username)
);
CREATE UNIQUE INDEX id_idx ON users (id);

insert into users (email, username, password) values
('asdf@asdf.com', 'a', 'asdf');


-- INSERT INTO users (email, username, pw)
-- SELECT $1, $2, $3
-- WHERE NOT EXISTS (SELECT email FROM users WHERE email = $1);

-- INSERT INTO users (email, username, pw)
-- SELECT 'bb', 'bb', 'bb'
-- WHERE NOT EXISTS (SELECT email FROM users WHERE email = 'bb');