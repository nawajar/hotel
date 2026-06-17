create table bookings (
    id uuid primary key default gen_random_uuid(),
    booking_ref text not null unique,
    status text not null default 'active' check (status in ('active','cancelled')),
    payment_status text not null default 'unpaid' check (payment_status in ('paid','unpaid')),
    check_in timestamptz not null,
    check_out timestamptz not null,
    label text check (label in ('check_in','check_out','needs_attention')),
    note text,
    discount_type text check (discount_type in ('amount','percentage')),
    discount_value float8 check (discount_value >= 0),
    customer_name text, customer_phone text, customer_id_type text, customer_id_number text,
    created_at timestamptz not null default now(),
    created_by uuid not null references users(id),
    updated_at timestamptz not null default now(),
    updated_by uuid not null references users(id),
    constraint check_dates check (check_out > check_in),
    constraint discount_both_or_neither check ((discount_type is null) = (discount_value is null))
);
create table booking_rooms (
    id uuid primary key default gen_random_uuid(),
    booking_id uuid not null references bookings(id) on delete cascade,
    room_id uuid not null references rooms(id),
    room_number text not null, room_name text not null,
    price_snapshot float8 not null check (price_snapshot >= 0),
    status text not null default 'active' check (status in ('active','cancelled')),
    updated_at timestamptz not null default now(),
    updated_by uuid not null references users(id)
);
create table booking_extra_services (
    id uuid primary key default gen_random_uuid(),
    booking_id uuid not null references bookings(id) on delete cascade,
    name text not null, amount float8 not null check (amount >= 0),
    created_at timestamptz not null default now(),
    updated_by uuid not null references users(id)
);
create index on bookings (check_in);
create index on bookings (status);
create index on booking_rooms (booking_id);
