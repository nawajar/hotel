create table booking_documents (
    id          uuid primary key default gen_random_uuid(),
    booking_id  uuid not null references bookings(id) on delete cascade,
    filename    text not null,
    stored_name text not null,
    mime_type   text not null,
    size        bigint not null,
    uploaded_by uuid not null references users(id),
    created_at  timestamptz not null default now()
);

create index booking_documents_booking_id_idx on booking_documents(booking_id);
