-- Hotel rooms available for booking, with pricing and active/inactive status.
create table rooms
(
    id          uuid        primary key default gen_random_uuid(),
    room_number text        not null unique,
    room_name   text        not null,
    is_active   boolean     not null default true,
    price       numeric(10, 2) not null check (price >= 0),
    updated_at  timestamptz not null default now(),
    updated_by  uuid        not null references users(id)
);
