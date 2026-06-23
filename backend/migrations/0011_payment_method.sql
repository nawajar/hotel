alter table bookings add column payment_method text check (payment_method in ('cash', 'bank_transfer'));
alter table booking_documents add column category text check (category in ('evidence', 'general')) not null default 'general';
