-- Your SQL goes here
create table if not exists files (
             id integer not null primary key,
             name text not null unique,
             status integer not null,
             frozen integer,
             sha2 text not null,
             last_update timestamp
         )