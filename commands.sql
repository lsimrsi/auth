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
('a', 'a', '$argon2i$v=19$m=4096,t=3,p=1$cmFuZG9tK3NhbHQ$5gYGvSfsiNtuQ1hjAQMf1xlU9rjfFSuLGcb/eB95xjg');

CREATE PROCEDURE add_user(e text, u text, p text)
LANGUAGE SQL
AS $$
INSERT INTO users (email, username, password) VALUES (e, u, p)
$$;

CALL add_user('c@c.com', 'c', 'c');
CALL add_user('d@d.dom', 'd', 'd');
-- INSERT INTO users (email, username, pw)
-- SELECT $1, $2, $3
-- WHERE NOT EXISTS (SELECT email FROM users WHERE email = $1);

-- INSERT INTO users (email, username, pw)
-- SELECT 'bb', 'bb', 'bb'
-- WHERE NOT EXISTS (SELECT email FROM users WHERE email = 'bb');