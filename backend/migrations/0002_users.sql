create extension if not exists citext;

create table users
(
    id            uuid primary key default gen_random_uuid(),
    email         citext not null unique,
    name          text not null,
    password_hash text not null,
    role          text not null check (role in ('admin', 'employee')),
    created_at    timestamptz not null default now(),
    updated_at    timestamptz not null default now()
);
