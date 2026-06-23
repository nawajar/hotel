alter table bookings add column deposit_amount float8;
alter table bookings add column deposit_returned boolean not null default false;
alter table bookings add column deposit_returned_at timestamptz;
alter table bookings add column deposit_returned_by uuid references users(id);
alter table bookings add constraint deposit_amount_positive check (deposit_amount is null or deposit_amount >= 0);
