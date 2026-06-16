-- Schema/table expected by tower-sessions-sqlx-store's PostgresStore
-- (default schema_name "tower_sessions", table_name "session").
-- Defined explicitly here so `sqlx migrate run` owns the full schema;
-- the store is configured to NOT run its own auto-migration.
create schema if not exists "tower_sessions";

create table if not exists "tower_sessions"."session"
(
    id text primary key not null,
    data bytea not null,
    expiry_date timestamptz not null
);
