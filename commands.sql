drop table if exists users;
create table users (
    id serial primary key,
    email text not null,
    username text not null,
    pw text not null,
    UNIQUE (email, username)
);
CREATE UNIQUE INDEX id_idx ON users (id);

insert into users (email, username, pw) values
('asdf@asdf.com', 'a', 'asdf');