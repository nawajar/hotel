alter table bookings add column paid_at timestamptz;
update bookings set paid_at = updated_at where payment_status = 'paid';
