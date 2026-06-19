create table notes (
    id         uuid        primary key default gen_random_uuid(),
    user_id    uuid        not null references users(id) on delete cascade,
    body       text        not null,
    done       boolean     not null default false,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index notes_user_id_idx on notes(user_id);
