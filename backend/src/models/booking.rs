use serde::Serialize;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::room::Room;

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
    pub needs_attention: i64,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct BookingDetail {
    pub booking: Booking,
    pub rooms: Vec<BookingRoom>,
    pub extra_services: Vec<BookingExtraService>,
    pub documents: Vec<BookingDocument>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IncomeSummaryRow {
    pub period: String,
    pub booking_count: i64,
    pub room_revenue: f64,
    pub extra_revenue: f64,
    pub discount_total: f64,
    pub net_revenue: f64,
}

impl Booking {
    pub async fn list_all(pool: &PgPool) -> Result<Vec<Booking>, sqlx::Error> {
        sqlx::query_as!(
            Booking,
            r#"select b.id, b.booking_ref, b.status, b.payment_status,
                      b.check_in, b.check_out, b.label, b.note,
                      b.discount_type, b.discount_value,
                      b.customer_name, b.customer_phone, b.customer_id_type, b.customer_id_number,
                      b.created_at, b.created_by, b.updated_at, b.updated_by,
                      coalesce(uc.name, '') as "created_by_name!",
                      coalesce(uu.name, '') as "updated_by_name!"
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
            r#"select b.id, b.booking_ref, b.status, b.payment_status,
                      b.check_in, b.check_out, b.label, b.note,
                      b.discount_type, b.discount_value,
                      b.customer_name, b.customer_phone, b.customer_id_type, b.customer_id_number,
                      b.created_at, b.created_by, b.updated_at, b.updated_by,
                      coalesce(uc.name, '') as "created_by_name!",
                      coalesce(uu.name, '') as "updated_by_name!"
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
        label: Option<&str>,
        note: Option<&str>,
        discount_type: Option<&str>,
        discount_value: Option<f64>,
        payment_status: &str,
        customer_name: Option<&str>,
        customer_phone: Option<&str>,
        customer_id_type: Option<&str>,
        customer_id_number: Option<&str>,
        created_by: Uuid,
    ) -> Result<BookingDetail, sqlx::Error> {
        let mut tx = pool.begin().await?;

        let booking_id = sqlx::query_scalar!(
            r#"insert into bookings
                   (booking_ref, check_in, check_out, label, note,
                    discount_type, discount_value, payment_status,
                    customer_name, customer_phone, customer_id_type, customer_id_number,
                    created_by, updated_by)
               values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)
               returning id"#,
            booking_ref,
            check_in,
            check_out,
            label,
            note,
            discount_type,
            discount_value,
            payment_status,
            customer_name,
            customer_phone,
            customer_id_type,
            customer_id_number,
            created_by
        )
        .fetch_one(&mut *tx)
        .await?;

        for room_id in room_ids {
            let room = sqlx::query_as!(
                Room,
                "select id, room_number, room_name, room_description, is_active, price, updated_at, updated_by from rooms where id = $1",
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
        label: Option<&str>,
        note: Option<&str>,
        discount_type: Option<&str>,
        discount_value: Option<f64>,
        payment_status: &str,
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
                   label           = $4,
                   note            = $5,
                   discount_type   = $6,
                   discount_value  = $7,
                   payment_status  = $8,
                   customer_name   = $9,
                   customer_phone  = $10,
                   customer_id_type    = $11,
                   customer_id_number  = $12,
                   updated_by      = $13,
                   updated_at      = now()
               where id = $1"#,
            id,
            check_in,
            check_out,
            label,
            note,
            discount_type,
            discount_value,
            payment_status,
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
                     select b.id, b.check_in, b.payment_status, b.discount_type, b.discount_value,
                            coalesce(rt.room_rev,0) as room_rev, coalesce(et.extra_rev,0) as extra_rev
                     from bookings b
                     left join room_totals rt on rt.booking_id=b.id
                     left join extra_totals et on et.booking_id=b.id
                     where b.status='active' and extract(year from b.check_in)=$1
                       and extract(month from b.check_in)=$2
                   )
                   select
                     to_char(check_in,'YYYY-MM-DD') as "period!",
                     count(*)::int8 as "booking_count!",
                     coalesce(sum(case when payment_status='paid' then room_rev else 0 end),0)::float8 as "room_revenue!",
                     coalesce(sum(case when payment_status='paid' then extra_rev else 0 end),0)::float8 as "extra_revenue!",
                     coalesce(sum(case when payment_status='paid' then
                        case discount_type
                          when 'amount' then least(discount_value, room_rev+extra_rev)
                          when 'percentage' then (room_rev+extra_rev)*discount_value/100.0
                          else 0 end
                      else 0 end),0)::float8 as "discount_total!",
                     coalesce(sum(case when payment_status='paid' then
                        (room_rev+extra_rev) -
                        case discount_type
                          when 'amount' then least(discount_value, room_rev+extra_rev)
                          when 'percentage' then (room_rev+extra_rev)*discount_value/100.0
                          else 0 end
                      else 0 end),0)::float8 as "net_revenue!"
                   from booking_totals
                   group by to_char(check_in,'YYYY-MM-DD')
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
                     select b.id, b.check_in, b.payment_status, b.discount_type, b.discount_value,
                            coalesce(rt.room_rev,0) as room_rev, coalesce(et.extra_rev,0) as extra_rev
                     from bookings b
                     left join room_totals rt on rt.booking_id=b.id
                     left join extra_totals et on et.booking_id=b.id
                     where b.status='active' and extract(year from b.check_in)=$1
                   )
                   select
                     to_char(check_in,'YYYY-MM') as "period!",
                     count(*)::int8 as "booking_count!",
                     coalesce(sum(case when payment_status='paid' then room_rev else 0 end),0)::float8 as "room_revenue!",
                     coalesce(sum(case when payment_status='paid' then extra_rev else 0 end),0)::float8 as "extra_revenue!",
                     coalesce(sum(case when payment_status='paid' then
                        case discount_type
                          when 'amount' then least(discount_value, room_rev+extra_rev)
                          when 'percentage' then (room_rev+extra_rev)*discount_value/100.0
                          else 0 end
                      else 0 end),0)::float8 as "discount_total!",
                     coalesce(sum(case when payment_status='paid' then
                        (room_rev+extra_rev) -
                        case discount_type
                          when 'amount' then least(discount_value, room_rev+extra_rev)
                          when 'percentage' then (room_rev+extra_rev)*discount_value/100.0
                          else 0 end
                      else 0 end),0)::float8 as "net_revenue!"
                   from booking_totals
                   group by to_char(check_in,'YYYY-MM')
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
                     and b.check_in  <= now()
                     and b.check_out > now()
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

        let needs_attention = sqlx::query_scalar!(
            "select count(*) from bookings where status = 'active' and label = 'needs_attention'"
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
            needs_attention,
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
                      d.created_at
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
                      d.created_at
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
    ) -> Result<BookingDocument, sqlx::Error> {
        let id = sqlx::query_scalar!(
            r#"insert into booking_documents (booking_id, filename, stored_name, mime_type, size, uploaded_by)
               values ($1, $2, $3, $4, $5, $6)
               returning id"#,
            booking_id,
            filename,
            stored_name,
            mime_type,
            size,
            uploaded_by
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
