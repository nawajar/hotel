use serde::Serialize;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
pub struct CalendarBookingRoom {
    pub id: Uuid,
    pub room_number: String,
    pub room_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalendarBooking {
    pub id: Uuid,
    pub booking_ref: String,
    pub status: String,
    pub payment_status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub check_in: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub check_out: OffsetDateTime,
    pub customer_name: Option<String>,
    pub note: Option<String>,
    pub rooms: Vec<CalendarBookingRoom>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoomAvailability {
    pub id: Uuid,
    pub room_number: String,
    pub room_name: String,
    pub price: f64,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TodaySummary {
    pub total_rooms: i64,
    pub available_now: i64,
    pub occupied_now: i64,
    pub check_ins_today: i64,
    pub check_outs_today: i64,
    pub unpaid_active: i64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct BookingRoom {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub room_id: Uuid,
    pub room_number: String,
    pub room_name: String,
    pub price_snapshot: f64,
    pub status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub updated_by: Uuid,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct BookingExtraService {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub name: String,
    pub amount: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub updated_by: Uuid,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Booking {
    pub id: Uuid,
    pub booking_ref: String,
    pub status: String,
    pub payment_status: String,
    pub payment_method: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub check_in: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub check_out: OffsetDateTime,
    pub label: Option<String>,
    pub note: Option<String>,
    pub discount_type: Option<String>,
    pub discount_value: Option<f64>,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub customer_id_type: Option<String>,
    pub customer_id_number: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub created_by: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub updated_by: Uuid,
    pub created_by_name: String,
    pub updated_by_name: String,
    pub deposit_amount: Option<f64>,
    pub deposit_returned: bool,
    #[serde(
        with = "time::serde::rfc3339::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub deposit_returned_at: Option<OffsetDateTime>,
    pub deposit_returned_by: Option<Uuid>,
    #[serde(
        with = "time::serde::rfc3339::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub paid_at: Option<OffsetDateTime>,
    #[serde(
        with = "time::serde::rfc3339::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub actual_check_in: Option<OffsetDateTime>,
    #[serde(
        with = "time::serde::rfc3339::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub actual_check_out: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct BookingDocument {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub filename: String,
    #[serde(skip)]
    pub stored_name: String,
    pub mime_type: String,
    pub size: i64,
    pub uploaded_by: Uuid,
    pub uploaded_by_name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub category: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BookingDetail {
    pub booking: Booking,
    pub rooms: Vec<BookingRoom>,
    pub extra_services: Vec<BookingExtraService>,
    pub documents: Vec<BookingDocument>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct IncomeSummaryRow {
    pub period: String,
    pub booking_count: i64,
    pub room_revenue: f64,
    pub extra_revenue: f64,
    pub discount_total: f64,
    pub net_revenue: f64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct IncomeDetailRow {
    pub booking_ref: String,
    pub customer_name: Option<String>,
    pub paid_date: String,
    pub room_revenue: f64,
    pub extra_revenue: f64,
    pub discount_total: f64,
    pub net_revenue: f64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct DailyReportRow {
    pub date: String,
    pub booking_count: i64,
    pub cash_revenue: f64,
    pub bank_transfer_revenue: f64,
    pub unspecified_revenue: f64,
    pub total_revenue: f64,
    pub cumulative_total: f64,
    pub deposit_held_count: i64,
    pub deposit_held_amount: f64,
    pub unpaid_count: i64,
    pub unpaid_amount: f64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct DailyReportDetailRow {
    pub record_type: String, // "paid" | "unpaid"
    pub booking_ref: String,
    pub customer_name: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub check_in: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub check_out: OffsetDateTime,
    pub payment_method: Option<String>,
    pub rooms: String,
    pub net_revenue: f64,
    pub deposit_amount: Option<f64>,
    pub deposit_returned: bool,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct DepositSummary {
    pub count: i64,
    pub total_amount: f64,
}

impl Booking {
    pub async fn calendar_list(
        pool: &PgPool,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<CalendarBooking>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"select b.id, b.booking_ref, b.status, b.payment_status,
                      b.check_in, b.check_out, b.customer_name, b.note,
                      br.room_id as "room_id?", br.room_number as "room_number?", br.room_name as "room_name?"
               from bookings b
               left join booking_rooms br on br.booking_id = b.id and br.status = 'active'
               where b.status = 'active' and b.check_in <= $2 and b.check_out >= $1
               order by b.check_in asc, b.id asc, br.room_number asc"#,
            start,
            end,
        )
        .fetch_all(pool)
        .await?;

        let mut result: Vec<CalendarBooking> = Vec::new();
        let mut last_id: Option<Uuid> = None;

        for row in rows {
            if last_id != Some(row.id) {
                result.push(CalendarBooking {
                    id: row.id,
                    booking_ref: row.booking_ref,
                    status: row.status,
                    payment_status: row.payment_status,
                    check_in: row.check_in,
                    check_out: row.check_out,
                    customer_name: row.customer_name,
                    note: row.note,
                    rooms: Vec::new(),
                });
                last_id = Some(row.id);
            }
            if let (Some(room_id), Some(room_number), Some(room_name)) =
                (row.room_id, row.room_number, row.room_name)
            {
                if let Some(booking) = result.last_mut() {
                    booking.rooms.push(CalendarBookingRoom {
                        id: room_id,
                        room_number,
                        room_name,
                    });
                }
            }
        }

        Ok(result)
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<Booking>, sqlx::Error> {
        sqlx::query_as!(
            Booking,
            r#"select b.id, b.booking_ref, b.status, b.payment_status, b.payment_method,
                      b.check_in, b.check_out, b.label, b.note,
                      b.discount_type, b.discount_value,
                      b.customer_name, b.customer_phone, b.customer_id_type, b.customer_id_number,
                      b.created_at, b.created_by, b.updated_at, b.updated_by,
                      coalesce(uc.name, '') as "created_by_name!",
                      coalesce(uu.name, '') as "updated_by_name!",
                      b.deposit_amount, b.deposit_returned, b.deposit_returned_at, b.deposit_returned_by,
                      b.paid_at, b.actual_check_in, b.actual_check_out
               from bookings b
               left join users uc on uc.id = b.created_by
               left join users uu on uu.id = b.updated_by
               order by b.check_in desc"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<BookingDetail>, sqlx::Error> {
        let booking = sqlx::query_as!(
            Booking,
            r#"select b.id, b.booking_ref, b.status, b.payment_status, b.payment_method,
                      b.check_in, b.check_out, b.label, b.note,
                      b.discount_type, b.discount_value,
                      b.customer_name, b.customer_phone, b.customer_id_type, b.customer_id_number,
                      b.created_at, b.created_by, b.updated_at, b.updated_by,
                      coalesce(uc.name, '') as "created_by_name!",
                      coalesce(uu.name, '') as "updated_by_name!",
                      b.deposit_amount, b.deposit_returned, b.deposit_returned_at, b.deposit_returned_by,
                      b.paid_at, b.actual_check_in, b.actual_check_out
               from bookings b
               left join users uc on uc.id = b.created_by
               left join users uu on uu.id = b.updated_by
               where b.id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await?;

        let Some(booking) = booking else {
            return Ok(None);
        };

        let rooms = sqlx::query_as!(
            BookingRoom,
            r#"select id, booking_id, room_id, room_number, room_name,
                      price_snapshot, status, updated_at, updated_by
               from booking_rooms
               where booking_id = $1
               order by id asc"#,
            id
        )
        .fetch_all(pool)
        .await?;

        let extra_services = sqlx::query_as!(
            BookingExtraService,
            r#"select id, booking_id, name, amount, created_at, updated_by
               from booking_extra_services
               where booking_id = $1
               order by created_at asc"#,
            id
        )
        .fetch_all(pool)
        .await?;

        let documents = BookingDocument::list_for_booking(pool, id).await?;

        Ok(Some(BookingDetail { booking, rooms, extra_services, documents }))
    }

    pub async fn count_existing_rooms(
        pool: &PgPool,
        room_ids: &[Uuid],
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!("select count(*) from rooms where id = any($1)", room_ids)
            .fetch_one(pool)
            .await
            .map(|c| c.unwrap_or(0))
    }

    /// Returns room numbers that are already booked during the given period.
    /// Pass `exclude_booking_id = Uuid::nil()` when creating (no booking to exclude).
    /// Pass the booking's own ID when updating (so it doesn't conflict with itself).
    pub async fn find_overlapping_rooms(
        pool: &PgPool,
        room_ids: &[Uuid],
        check_in: OffsetDateTime,
        check_out: OffsetDateTime,
        exclude_booking_id: Uuid,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"select distinct br.room_number
               from booking_rooms br
               join bookings b on b.id = br.booking_id
               where br.room_id = any($1)
                 and br.status = 'active'
                 and b.status  = 'active'
                 and b.check_in  < $3
                 and b.check_out > $2
                 and b.id != $4
               order by br.room_number"#,
            room_ids,
            check_in,
            check_out,
            exclude_booking_id,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.room_number).collect())
    }

    pub async fn active_room_ids(pool: &PgPool, booking_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
        let rows = sqlx::query!(
            "select room_id from booking_rooms where booking_id = $1 and status = 'active'",
            booking_id
        )
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.room_id).collect())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &PgPool,
        check_in: OffsetDateTime,
        check_out: OffsetDateTime,
        room_ids: &[Uuid],
        booking_ref: &str,
        note: Option<&str>,
        discount_type: Option<&str>,
        discount_value: Option<f64>,
        payment_status: &str,
        payment_method: Option<&str>,
        deposit_amount: Option<f64>,
        customer_name: Option<&str>,
        customer_phone: Option<&str>,
        customer_id_type: Option<&str>,
        customer_id_number: Option<&str>,
        created_by: Uuid,
    ) -> Result<BookingDetail, sqlx::Error> {
        let mut tx = pool.begin().await?;

        let booking_id = sqlx::query_scalar!(
            r#"insert into bookings
                   (booking_ref, check_in, check_out, note,
                    discount_type, discount_value, payment_status, paid_at,
                    payment_method, deposit_amount,
                    customer_name, customer_phone, customer_id_type, customer_id_number,
                    created_by, updated_by)
               values ($1, $2, $3, $4, $5, $6, $7,
                       case when $7 = 'paid' then now() else null end,
                       $8, $9,
                       $10, $11, $12, $13, $14, $14)
               returning id"#,
            booking_ref,
            check_in,
            check_out,
            note,
            discount_type,
            discount_value,
            payment_status,
            payment_method,
            deposit_amount,
            customer_name,
            customer_phone,
            customer_id_type,
            customer_id_number,
            created_by
        )
        .fetch_one(&mut *tx)
        .await?;

        for room_id in room_ids {
            let room = sqlx::query!(
                "select id, room_number, room_name, price from rooms where id = $1",
                room_id
            )
            .fetch_optional(&mut *tx)
            .await?;

            let Some(room) = room else {
                return Err(sqlx::Error::RowNotFound);
            };

            sqlx::query!(
                "insert into booking_rooms (booking_id, room_id, room_number, room_name, price_snapshot, updated_by) values ($1, $2, $3, $4, $5, $6)",
                booking_id,
                room.id,
                room.room_number,
                room.room_name,
                room.price,
                created_by
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Self::find_by_id(pool, booking_id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        check_in: OffsetDateTime,
        check_out: OffsetDateTime,
        note: Option<&str>,
        discount_type: Option<&str>,
        discount_value: Option<f64>,
        payment_status: &str,
        payment_method: Option<&str>,
        deposit_amount: Option<f64>,
        customer_name: Option<&str>,
        customer_phone: Option<&str>,
        customer_id_type: Option<&str>,
        customer_id_number: Option<&str>,
        updated_by: Uuid,
    ) -> Result<Option<BookingDetail>, sqlx::Error> {
        let rows_affected = sqlx::query!(
            r#"update bookings
               set check_in        = $2,
                   check_out       = $3,
                   note            = $4,
                   discount_type   = $5,
                   discount_value  = $6,
                   payment_status  = $7,
                   paid_at         = case when $7 = 'paid' then coalesce(paid_at, now()) else null end,
                   payment_method  = $8,
                   deposit_amount  = $9,
                   customer_name   = $10,
                   customer_phone  = $11,
                   customer_id_type    = $12,
                   customer_id_number  = $13,
                   updated_by      = $14,
                   updated_at      = now()
               where id = $1"#,
            id,
            check_in,
            check_out,
            note,
            discount_type,
            discount_value,
            payment_status,
            payment_method,
            deposit_amount,
            customer_name,
            customer_phone,
            customer_id_type,
            customer_id_number,
            updated_by
        )
        .execute(pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Ok(None);
        }

        Self::find_by_id(pool, id).await
    }

    pub async fn return_deposit(
        pool: &PgPool,
        booking_id: Uuid,
        returned_by: Uuid,
    ) -> Result<Option<BookingDetail>, sqlx::Error> {
        let rows_affected = sqlx::query!(
            r#"update bookings
               set deposit_returned    = true,
                   deposit_returned_at = now(),
                   deposit_returned_by = $2,
                   updated_by          = $2,
                   updated_at          = now()
               where id = $1 and deposit_returned = false"#,
            booking_id,
            returned_by
        )
        .execute(pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            // Either not found or already returned — fetch anyway to disambiguate at handler level
            return Self::find_by_id(pool, booking_id).await;
        }

        Self::find_by_id(pool, booking_id).await
    }

    pub async fn record_check_in(
        pool: &PgPool,
        booking_id: Uuid,
        updated_by: Uuid,
    ) -> Result<Option<BookingDetail>, AppError> {
        let rows_affected = sqlx::query!(
            r#"update bookings
               set actual_check_in = now(),
                   updated_by      = $2,
                   updated_at      = now()
               where id = $1 and status = 'active' and actual_check_in is null"#,
            booking_id,
            updated_by
        )
        .execute(pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            let existing = Self::find_by_id(pool, booking_id).await?;
            if existing.is_none() {
                return Ok(None);
            }
            return Err(AppError::Conflict("booking cannot be checked in".into()));
        }

        Ok(Self::find_by_id(pool, booking_id).await?)
    }

    pub async fn record_check_out(
        pool: &PgPool,
        booking_id: Uuid,
        updated_by: Uuid,
    ) -> Result<Option<BookingDetail>, AppError> {
        let rows_affected = sqlx::query!(
            r#"update bookings
               set actual_check_out = now(),
                   updated_by       = $2,
                   updated_at       = now()
               where id = $1 and status = 'active' and actual_check_in is not null and actual_check_out is null"#,
            booking_id,
            updated_by
        )
        .execute(pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            let existing = Self::find_by_id(pool, booking_id).await?;
            if existing.is_none() {
                return Ok(None);
            }
            return Err(AppError::Conflict(
                "booking cannot be checked out before check-in".into(),
            ));
        }

        Ok(Self::find_by_id(pool, booking_id).await?)
    }

    pub async fn cancel(
        pool: &PgPool,
        id: Uuid,
        updated_by: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let mut tx = pool.begin().await?;

        let rows_affected = sqlx::query!(
            r#"update bookings
               set status     = 'cancelled',
                   updated_by = $2,
                   updated_at = now()
               where id = $1"#,
            id,
            updated_by
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if rows_affected > 0 {
            sqlx::query!(
                "update booking_rooms set status = 'cancelled', updated_by = $2, updated_at = now() where booking_id = $1 and status = 'active'",
                id,
                updated_by
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(rows_affected > 0)
    }

    pub async fn cancel_room(
        pool: &PgPool,
        booking_id: Uuid,
        room_booking_id: Uuid,
        updated_by: Uuid,
    ) -> Result<Option<BookingRoom>, sqlx::Error> {
        sqlx::query_as!(
            BookingRoom,
            r#"update booking_rooms
               set status     = 'cancelled',
                   updated_by = $3,
                   updated_at = now()
               where id = $1 and booking_id = $2
               returning id, booking_id, room_id, room_number, room_name,
                         price_snapshot, status, updated_at, updated_by"#,
            room_booking_id,
            booking_id,
            updated_by
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn add_extra_service(
        pool: &PgPool,
        booking_id: Uuid,
        name: &str,
        amount: f64,
        updated_by: Uuid,
    ) -> Result<BookingExtraService, sqlx::Error> {
        sqlx::query_as!(
            BookingExtraService,
            r#"insert into booking_extra_services (booking_id, name, amount, updated_by)
               values ($1, $2, $3, $4)
               returning id, booking_id, name, amount, created_at, updated_by"#,
            booking_id,
            name,
            amount,
            updated_by
        )
        .fetch_one(pool)
        .await
    }

    pub async fn remove_extra_service(
        pool: &PgPool,
        booking_id: Uuid,
        service_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "delete from booking_extra_services where id = $1 and booking_id = $2",
            service_id,
            booking_id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn income_summary(
        pool: &PgPool,
        year: i32,
        month: Option<i32>,
    ) -> Result<Vec<IncomeSummaryRow>, sqlx::Error> {
        if let Some(month) = month {
            let rows = sqlx::query!(
                r#"with room_totals as (
                     select br.booking_id, sum(br.price_snapshot) as room_rev
                     from booking_rooms br where br.status='active' group by br.booking_id
                   ),
                   extra_totals as (
                     select bes.booking_id, sum(bes.amount) as extra_rev
                     from booking_extra_services bes group by bes.booking_id
                   ),
                   booking_totals as (
                     select b.id, b.paid_at, b.discount_type, b.discount_value,
                            coalesce(rt.room_rev,0) as room_rev, coalesce(et.extra_rev,0) as extra_rev
                     from bookings b
                     left join room_totals rt on rt.booking_id=b.id
                     left join extra_totals et on et.booking_id=b.id
                     where b.status='active' and b.paid_at is not null
                       and extract(year from b.paid_at)=$1
                       and extract(month from b.paid_at)=$2
                   )
                   select
                     to_char(paid_at,'YYYY-MM-DD') as "period!",
                     count(*)::int8 as "booking_count!",
                     coalesce(sum(room_rev),0)::float8 as "room_revenue!",
                     coalesce(sum(extra_rev),0)::float8 as "extra_revenue!",
                     coalesce(sum(
                       case discount_type
                         when 'amount' then least(discount_value, room_rev+extra_rev)
                         when 'percentage' then (room_rev+extra_rev)*discount_value/100.0
                         else 0 end
                     ),0)::float8 as "discount_total!",
                     coalesce(sum(
                       (room_rev+extra_rev) -
                       case discount_type
                         when 'amount' then least(discount_value, room_rev+extra_rev)
                         when 'percentage' then (room_rev+extra_rev)*discount_value/100.0
                         else 0 end
                     ),0)::float8 as "net_revenue!"
                   from booking_totals
                   group by to_char(paid_at,'YYYY-MM-DD')
                   order by "period!" asc"#,
                year as i64,
                month as i64
            )
            .fetch_all(pool)
            .await?;

            Ok(rows
                .into_iter()
                .map(|r| IncomeSummaryRow {
                    period: r.period,
                    booking_count: r.booking_count,
                    room_revenue: r.room_revenue,
                    extra_revenue: r.extra_revenue,
                    discount_total: r.discount_total,
                    net_revenue: r.net_revenue,
                })
                .collect())
        } else {
            let rows = sqlx::query!(
                r#"with room_totals as (
                     select br.booking_id, sum(br.price_snapshot) as room_rev
                     from booking_rooms br where br.status='active' group by br.booking_id
                   ),
                   extra_totals as (
                     select bes.booking_id, sum(bes.amount) as extra_rev
                     from booking_extra_services bes group by bes.booking_id
                   ),
                   booking_totals as (
                     select b.id, b.paid_at, b.discount_type, b.discount_value,
                            coalesce(rt.room_rev,0) as room_rev, coalesce(et.extra_rev,0) as extra_rev
                     from bookings b
                     left join room_totals rt on rt.booking_id=b.id
                     left join extra_totals et on et.booking_id=b.id
                     where b.status='active' and b.paid_at is not null
                       and extract(year from b.paid_at)=$1
                   )
                   select
                     to_char(paid_at,'YYYY-MM') as "period!",
                     count(*)::int8 as "booking_count!",
                     coalesce(sum(room_rev),0)::float8 as "room_revenue!",
                     coalesce(sum(extra_rev),0)::float8 as "extra_revenue!",
                     coalesce(sum(
                       case discount_type
                         when 'amount' then least(discount_value, room_rev+extra_rev)
                         when 'percentage' then (room_rev+extra_rev)*discount_value/100.0
                         else 0 end
                     ),0)::float8 as "discount_total!",
                     coalesce(sum(
                       (room_rev+extra_rev) -
                       case discount_type
                         when 'amount' then least(discount_value, room_rev+extra_rev)
                         when 'percentage' then (room_rev+extra_rev)*discount_value/100.0
                         else 0 end
                     ),0)::float8 as "net_revenue!"
                   from booking_totals
                   group by to_char(paid_at,'YYYY-MM')
                   order by "period!" asc"#,
                year as i64
            )
            .fetch_all(pool)
            .await?;

            Ok(rows
                .into_iter()
                .map(|r| IncomeSummaryRow {
                    period: r.period,
                    booking_count: r.booking_count,
                    room_revenue: r.room_revenue,
                    extra_revenue: r.extra_revenue,
                    discount_total: r.discount_total,
                    net_revenue: r.net_revenue,
                })
                .collect())
        }
    }

    pub async fn income_detail(
        pool: &PgPool,
        year: i32,
        month: i32,
    ) -> Result<Vec<IncomeDetailRow>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"with room_totals as (
                 select br.booking_id, sum(br.price_snapshot) as room_rev
                 from booking_rooms br where br.status='active' group by br.booking_id
               ),
               extra_totals as (
                 select bes.booking_id, sum(bes.amount) as extra_rev
                 from booking_extra_services bes group by bes.booking_id
               )
               select
                 b.booking_ref as "booking_ref!",
                 b.customer_name,
                 to_char(b.paid_at,'YYYY-MM-DD') as "paid_date!",
                 coalesce(rt.room_rev,0)::float8 as "room_revenue!",
                 coalesce(et.extra_rev,0)::float8 as "extra_revenue!",
                 coalesce(
                   case b.discount_type
                     when 'amount' then least(b.discount_value, coalesce(rt.room_rev,0)+coalesce(et.extra_rev,0))
                     when 'percentage' then (coalesce(rt.room_rev,0)+coalesce(et.extra_rev,0))*b.discount_value/100.0
                     else 0 end
                 ,0)::float8 as "discount_total!",
                 coalesce(
                   (coalesce(rt.room_rev,0)+coalesce(et.extra_rev,0)) -
                   case b.discount_type
                     when 'amount' then least(b.discount_value, coalesce(rt.room_rev,0)+coalesce(et.extra_rev,0))
                     when 'percentage' then (coalesce(rt.room_rev,0)+coalesce(et.extra_rev,0))*b.discount_value/100.0
                     else 0 end
                 ,0)::float8 as "net_revenue!"
               from bookings b
               left join room_totals rt on rt.booking_id=b.id
               left join extra_totals et on et.booking_id=b.id
               where b.status='active' and b.paid_at is not null
                 and extract(year from b.paid_at)=$1
                 and extract(month from b.paid_at)=$2
               order by b.paid_at asc, b.booking_ref asc"#,
            year as i64,
            month as i64
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| IncomeDetailRow {
                booking_ref: r.booking_ref,
                customer_name: r.customer_name,
                paid_date: r.paid_date,
                room_revenue: r.room_revenue,
                extra_revenue: r.extra_revenue,
                discount_total: r.discount_total,
                net_revenue: r.net_revenue,
            })
            .collect())
    }

    pub async fn daily_report(
        pool: &PgPool,
        year: i32,
        month: i32,
    ) -> Result<Vec<DailyReportRow>, sqlx::Error> {
        sqlx::query_as::<_, DailyReportRow>(
            r#"with room_totals as (
                 select br.booking_id, sum(br.price_snapshot) as room_rev
                 from booking_rooms br where br.status = 'active' group by br.booking_id
               ),
               extra_totals as (
                 select bes.booking_id, sum(bes.amount)::float8 as extra_rev
                 from booking_extra_services bes group by bes.booking_id
               ),
               net_amount as (
                 select
                   b.paid_at,
                   b.created_at,
                   b.payment_method,
                   b.deposit_amount,
                   b.deposit_returned,
                   (coalesce(rt.room_rev, 0) + coalesce(et.extra_rev, 0) -
                    case b.discount_type
                      when 'amount' then least(b.discount_value, coalesce(rt.room_rev, 0) + coalesce(et.extra_rev, 0))
                      when 'percentage' then (coalesce(rt.room_rev, 0) + coalesce(et.extra_rev, 0)) * b.discount_value / 100.0
                      else 0
                    end)::float8 as net_rev
                 from bookings b
                 left join room_totals rt on rt.booking_id = b.id
                 left join extra_totals et on et.booking_id = b.id
                 where b.status = 'active'
                   and (
                     (b.paid_at is not null
                       and extract(year from b.paid_at) = $1
                       and extract(month from b.paid_at) = $2)
                     or
                     (b.paid_at is null
                       and extract(year from b.created_at) = $1
                       and extract(month from b.created_at) = $2)
                   )
               ),
               paid_daily as (
                 select
                   to_char(paid_at, 'YYYY-MM-DD') as day,
                   count(*)::int8 as booking_count,
                   coalesce(sum(net_rev) filter (where payment_method = 'cash'), 0)::float8 as cash_revenue,
                   coalesce(sum(net_rev) filter (where payment_method = 'bank_transfer'), 0)::float8 as bank_transfer_revenue,
                   coalesce(sum(net_rev) filter (where payment_method is null), 0)::float8 as unspecified_revenue,
                   coalesce(sum(net_rev), 0)::float8 as total_revenue,
                   count(*) filter (where deposit_amount is not null and deposit_amount > 0 and deposit_returned = false)::int8 as deposit_held_count,
                   coalesce(sum(deposit_amount) filter (where deposit_amount is not null and deposit_amount > 0 and deposit_returned = false), 0)::float8 as deposit_held_amount
                 from net_amount
                 where paid_at is not null
                 group by day
               ),
               unpaid_daily as (
                 select
                   to_char(created_at, 'YYYY-MM-DD') as day,
                   count(*)::int8 as unpaid_count,
                   coalesce(sum(net_rev), 0)::float8 as unpaid_amount,
                   count(*) filter (where deposit_amount is not null and deposit_amount > 0 and deposit_returned = false)::int8 as deposit_held_count,
                   coalesce(sum(deposit_amount) filter (where deposit_amount is not null and deposit_amount > 0 and deposit_returned = false), 0)::float8 as deposit_held_amount
                 from net_amount
                 where paid_at is null
                 group by day
               ),
               all_days as (
                 select day from paid_daily
                 union
                 select day from unpaid_daily
               ),
               daily as (
                 select
                   a.day,
                   coalesce(p.booking_count, 0)::int8 as booking_count,
                   coalesce(p.cash_revenue, 0)::float8 as cash_revenue,
                   coalesce(p.bank_transfer_revenue, 0)::float8 as bank_transfer_revenue,
                   coalesce(p.unspecified_revenue, 0)::float8 as unspecified_revenue,
                   coalesce(p.total_revenue, 0)::float8 as total_revenue,
                   (coalesce(p.deposit_held_count, 0) + coalesce(u.deposit_held_count, 0))::int8 as deposit_held_count,
                   (coalesce(p.deposit_held_amount, 0) + coalesce(u.deposit_held_amount, 0))::float8 as deposit_held_amount,
                   coalesce(u.unpaid_count, 0)::int8 as unpaid_count,
                   coalesce(u.unpaid_amount, 0)::float8 as unpaid_amount
                 from all_days a
                 left join paid_daily p on p.day = a.day
                 left join unpaid_daily u on u.day = a.day
               )
               select
                 day as date,
                 booking_count,
                 cash_revenue,
                 bank_transfer_revenue,
                 unspecified_revenue,
                 total_revenue,
                 deposit_held_count,
                 deposit_held_amount,
                 unpaid_count,
                 unpaid_amount,
                 sum(total_revenue) over (order by day rows between unbounded preceding and current row)::float8 as cumulative_total
               from daily
               order by day asc"#,
        )
        .bind(year as i64)
        .bind(month as i64)
        .fetch_all(pool)
        .await
    }

    pub async fn daily_report_detail(
        pool: &PgPool,
        year: i32,
        month: i32,
        day: i32,
    ) -> Result<Vec<DailyReportDetailRow>, sqlx::Error> {
        sqlx::query_as::<_, DailyReportDetailRow>(
            r#"with bra as (
                 select br.booking_id,
                        string_agg(r.room_number, ', ' order by r.room_number) as rooms,
                        sum(br.price_snapshot) as room_rev
                 from booking_rooms br
                 join rooms r on r.id = br.room_id
                 where br.status = 'active'
                 group by br.booking_id
               ),
               ea as (
                 select booking_id, sum(amount)::float8 as extra_rev
                 from booking_extra_services
                 group by booking_id
               ),
               net as (
                 select
                   b.id, b.booking_ref, b.customer_name, b.check_in, b.check_out,
                   b.payment_method, b.paid_at, b.created_at,
                   b.deposit_amount, b.deposit_returned,
                   coalesce(bra.rooms, '') as rooms,
                   (coalesce(bra.room_rev, 0) + coalesce(ea.extra_rev, 0) -
                    case b.discount_type
                      when 'amount' then least(b.discount_value, coalesce(bra.room_rev, 0) + coalesce(ea.extra_rev, 0))
                      when 'percentage' then (coalesce(bra.room_rev, 0) + coalesce(ea.extra_rev, 0)) * b.discount_value / 100.0
                      else 0
                    end)::float8 as net_revenue
                 from bookings b
                 left join bra on bra.booking_id = b.id
                 left join ea on ea.booking_id = b.id
                 where b.status = 'active'
               )
               -- paid bookings: grouped by paid_at date
               select
                 'paid' as record_type,
                 booking_ref, customer_name, check_in, check_out,
                 payment_method, rooms, net_revenue, deposit_amount, deposit_returned
               from net
               where paid_at is not null
                 and extract(year from paid_at) = $1
                 and extract(month from paid_at) = $2
                 and extract(day from paid_at) = $3
               union all
               -- unpaid bookings: grouped by created_at date (still unpaid now)
               select
                 'unpaid' as record_type,
                 booking_ref, customer_name, check_in, check_out,
                 payment_method, rooms, net_revenue, deposit_amount, deposit_returned
               from net
               where paid_at is null
                 and extract(year from created_at) = $1
                 and extract(month from created_at) = $2
                 and extract(day from created_at) = $3
               order by record_type asc, check_in asc"#,
        )
        .bind(year as i64)
        .bind(month as i64)
        .bind(day as i64)
        .fetch_all(pool)
        .await
    }

    pub async fn unreturned_deposit_summary(pool: &PgPool) -> Result<DepositSummary, sqlx::Error> {
        sqlx::query_as::<_, DepositSummary>(
            r#"select
                 count(*)::int8 as count,
                 coalesce(sum(deposit_amount), 0)::float8 as total_amount
               from bookings
               where status = 'active'
                 and deposit_amount is not null
                 and deposit_amount > 0
                 and deposit_returned = false"#,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn room_availability(
        pool: &PgPool,
        check_in: OffsetDateTime,
        check_out: OffsetDateTime,
    ) -> Result<Vec<RoomAvailability>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"select r.id, r.room_number, r.room_name, r.price,
               case when exists (
                   select 1 from booking_rooms br
                   join bookings b on b.id = br.booking_id
                   where br.room_id = r.id
                     and br.status = 'active'
                     and b.status  = 'active'
                     and b.check_in  < $2
                     and b.check_out > $1
               ) then false else true end as "is_available!: bool"
               from rooms r
               where r.is_active = true
               order by r.room_number"#,
            check_in,
            check_out,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| RoomAvailability {
                id: r.id,
                room_number: r.room_number,
                room_name: r.room_name,
                price: r.price,
                is_available: r.is_available,
            })
            .collect())
    }

    pub async fn today_summary(pool: &PgPool) -> Result<TodaySummary, sqlx::Error> {
        let total_rooms = sqlx::query_scalar!(
            "select count(*) from rooms where is_active = true"
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let available_now = sqlx::query_scalar!(
            r#"select count(*) from rooms r where r.is_active = true
               and not exists (
                   select 1 from booking_rooms br
                   join bookings b on b.id = br.booking_id
                   where br.room_id = r.id
                     and br.status = 'active'
                     and b.status  = 'active'
                     and b.check_in::date  <= current_date
                     and b.check_out::date > current_date
               )"#
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let check_ins_today = sqlx::query_scalar!(
            "select count(*) from bookings where status = 'active' and check_in::date = current_date"
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let check_outs_today = sqlx::query_scalar!(
            "select count(*) from bookings where status = 'active' and check_out::date = current_date"
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let unpaid_active = sqlx::query_scalar!(
            "select count(*) from bookings where status = 'active' and payment_status = 'unpaid'"
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        Ok(TodaySummary {
            total_rooms,
            available_now,
            occupied_now: total_rooms - available_now,
            check_ins_today,
            check_outs_today,
            unpaid_active,
        })
    }

}

impl BookingDocument {
    pub async fn list_for_booking(
        pool: &PgPool,
        booking_id: Uuid,
    ) -> Result<Vec<BookingDocument>, sqlx::Error> {
        sqlx::query_as!(
            BookingDocument,
            r#"select d.id, d.booking_id, d.filename, d.stored_name, d.mime_type, d.size,
                      d.uploaded_by, coalesce(u.name, '') as "uploaded_by_name!",
                      d.created_at, d.category
               from booking_documents d
               left join users u on u.id = d.uploaded_by
               where d.booking_id = $1
               order by d.created_at asc"#,
            booking_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<BookingDocument>, sqlx::Error> {
        sqlx::query_as!(
            BookingDocument,
            r#"select d.id, d.booking_id, d.filename, d.stored_name, d.mime_type, d.size,
                      d.uploaded_by, coalesce(u.name, '') as "uploaded_by_name!",
                      d.created_at, d.category
               from booking_documents d
               left join users u on u.id = d.uploaded_by
               where d.id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        booking_id: Uuid,
        filename: &str,
        stored_name: &str,
        mime_type: &str,
        size: i64,
        uploaded_by: Uuid,
        category: &str,
    ) -> Result<BookingDocument, sqlx::Error> {
        let id = sqlx::query_scalar!(
            r#"insert into booking_documents (booking_id, filename, stored_name, mime_type, size, uploaded_by, category)
               values ($1, $2, $3, $4, $5, $6, $7)
               returning id"#,
            booking_id,
            filename,
            stored_name,
            mime_type,
            size,
            uploaded_by,
            category
        )
        .fetch_one(pool)
        .await?;

        Self::find_by_id(pool, id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Option<BookingDocument>, sqlx::Error> {
        let doc = Self::find_by_id(pool, id).await?;
        if doc.is_some() {
            sqlx::query!("delete from booking_documents where id = $1", id)
                .execute(pool)
                .await?;
        }
        Ok(doc)
    }
}
