use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::extractor::{AuthUser, RequireAdmin},
    error::AppError,
    models::booking::{
        Booking, BookingDetail, BookingDocument, BookingExtraService, BookingRoom,
        IncomeSummaryRow, RoomAvailability, TodaySummary,
    },
    AppState,
};

#[derive(Debug, Deserialize, Validate)]
struct CreateBookingInput {
    check_in: String,
    check_out: String,
    #[validate(length(min = 1))]
    room_ids: Vec<Uuid>,
    label: Option<String>,
    note: Option<String>,
    discount_type: Option<String>,
    discount_value: Option<f64>,
    payment_status: Option<String>,
    customer_name: Option<String>,
    customer_phone: Option<String>,
    customer_id_type: Option<String>,
    customer_id_number: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateBookingInput {
    check_in: String,
    check_out: String,
    label: Option<String>,
    note: Option<String>,
    discount_type: Option<String>,
    discount_value: Option<f64>,
    payment_status: Option<String>,
    customer_name: Option<String>,
    customer_phone: Option<String>,
    customer_id_type: Option<String>,
    customer_id_number: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
struct AddExtraServiceInput {
    #[validate(length(min = 1))]
    name: String,
    amount: f64,
}

#[derive(Debug, Deserialize)]
struct AvailabilityParams {
    check_in: String,
    check_out: String,
}

#[derive(Debug, Deserialize)]
struct IncomeSummaryParams {
    year: i32,
    month: Option<i32>,
}

fn parse_and_validate_dates(
    ci: &str,
    co: &str,
    enforce_future: bool,
) -> Result<(OffsetDateTime, OffsetDateTime), AppError> {
    let check_in = OffsetDateTime::parse(ci, &Rfc3339)
        .map_err(|_| AppError::Validation("check_in is not a valid RFC3339 datetime".into()))?;
    let check_out = OffsetDateTime::parse(co, &Rfc3339)
        .map_err(|_| AppError::Validation("check_out is not a valid RFC3339 datetime".into()))?;
    if enforce_future && check_in.date() < OffsetDateTime::now_utc().date() {
        return Err(AppError::Validation("check_in cannot be in the past".into()));
    }
    if check_out <= check_in {
        return Err(AppError::Validation("check_out must be after check_in".into()));
    }
    Ok((check_in, check_out))
}

fn validate_discount(dt: Option<&str>, dv: Option<f64>) -> Result<(), AppError> {
    match (dt, dv) {
        (None, None) => Ok(()),
        (Some(dt), Some(dv)) => {
            if dt != "amount" && dt != "percentage" {
                return Err(AppError::Validation(
                    "discount_type must be 'amount' or 'percentage'".into(),
                ));
            }
            if dv < 0.0 {
                return Err(AppError::Validation("discount_value must be >= 0".into()));
            }
            if dt == "percentage" && dv > 100.0 {
                return Err(AppError::Validation(
                    "discount_value must be <= 100 for percentage".into(),
                ));
            }
            Ok(())
        }
        _ => Err(AppError::Validation(
            "discount_type and discount_value must both be present or both absent".into(),
        )),
    }
}

fn validate_label(label: Option<&str>) -> Result<(), AppError> {
    if let Some(l) = label {
        if l != "check_in" && l != "check_out" && l != "needs_attention" {
            return Err(AppError::Validation(
                "label must be 'check_in', 'check_out', or 'needs_attention'".into(),
            ));
        }
    }
    Ok(())
}

fn validate_payment_status(ps: Option<&str>) -> Result<(), AppError> {
    if let Some(ps) = ps {
        if ps != "paid" && ps != "unpaid" {
            return Err(AppError::Validation(
                "payment_status must be 'paid' or 'unpaid'".into(),
            ));
        }
    }
    Ok(())
}

fn generate_booking_ref() -> String {
    format!(
        "BK-{}",
        Uuid::new_v4().to_string().replace('-', "")[..8].to_uppercase()
    )
}

async fn list_bookings(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Booking>>, AppError> {
    Ok(Json(Booking::list_all(&state.pool).await?))
}

async fn create_booking(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(input): Json<CreateBookingInput>,
) -> Result<(StatusCode, Json<BookingDetail>), AppError> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let (check_in, check_out) = parse_and_validate_dates(&input.check_in, &input.check_out, true)?;
    validate_label(input.label.as_deref())?;
    validate_discount(input.discount_type.as_deref(), input.discount_value)?;
    validate_payment_status(input.payment_status.as_deref())?;

    let payment_status = input.payment_status.as_deref().unwrap_or("unpaid");
    let booking_ref = generate_booking_ref();

    let existing = Booking::count_existing_rooms(&state.pool, &input.room_ids).await?;
    if existing != input.room_ids.len() as i64 {
        return Err(AppError::Validation("unknown room_id".into()));
    }

    let overlapping = Booking::find_overlapping_rooms(
        &state.pool,
        &input.room_ids,
        check_in,
        check_out,
        Uuid::nil(),
    )
    .await?;
    if !overlapping.is_empty() {
        return Err(AppError::Conflict(format!(
            "rooms already booked for this period: {}",
            overlapping.join(", ")
        )));
    }

    match Booking::create(
        &state.pool,
        check_in,
        check_out,
        &input.room_ids,
        &booking_ref,
        input.label.as_deref(),
        input.note.as_deref(),
        input.discount_type.as_deref(),
        input.discount_value,
        payment_status,
        input.customer_name.as_deref(),
        input.customer_phone.as_deref(),
        input.customer_id_type.as_deref(),
        input.customer_id_number.as_deref(),
        user.id,
    )
    .await
    {
        Ok(detail) => Ok((StatusCode::CREATED, Json(detail))),
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            Err(AppError::Conflict("booking_ref already in use".into()))
        }
        Err(e) => Err(e.into()),
    }
}

async fn get_booking(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BookingDetail>, AppError> {
    Booking::find_by_id(&state.pool, id)
        .await?
        .map(Json)
        .ok_or_else(|| AppError::NotFound("booking not found".into()))
}

async fn update_booking(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateBookingInput>,
) -> Result<Json<BookingDetail>, AppError> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let (check_in, check_out) = parse_and_validate_dates(&input.check_in, &input.check_out, false)?;
    validate_label(input.label.as_deref())?;
    validate_discount(input.discount_type.as_deref(), input.discount_value)?;
    validate_payment_status(input.payment_status.as_deref())?;

    let payment_status = input.payment_status.as_deref().unwrap_or("unpaid");

    let active_room_ids = Booking::active_room_ids(&state.pool, id).await?;
    if !active_room_ids.is_empty() {
        let overlapping = Booking::find_overlapping_rooms(
            &state.pool,
            &active_room_ids,
            check_in,
            check_out,
            id,
        )
        .await?;
        if !overlapping.is_empty() {
            return Err(AppError::Conflict(format!(
                "rooms already booked for this period: {}",
                overlapping.join(", ")
            )));
        }
    }

    Booking::update(
        &state.pool,
        id,
        check_in,
        check_out,
        input.label.as_deref(),
        input.note.as_deref(),
        input.discount_type.as_deref(),
        input.discount_value,
        payment_status,
        input.customer_name.as_deref(),
        input.customer_phone.as_deref(),
        input.customer_id_type.as_deref(),
        input.customer_id_number.as_deref(),
        user.id,
    )
    .await?
    .map(Json)
    .ok_or_else(|| AppError::NotFound("booking not found".into()))
}

async fn cancel_booking(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let found = Booking::cancel(&state.pool, id, user.id).await?;
    if found {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("booking not found".into()))
    }
}

async fn cancel_room(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path((id, room_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<BookingRoom>, AppError> {
    Booking::cancel_room(&state.pool, id, room_id, user.id)
        .await?
        .map(Json)
        .ok_or_else(|| AppError::NotFound("booking room not found".into()))
}

async fn add_extra_service(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<AddExtraServiceInput>,
) -> Result<(StatusCode, Json<BookingExtraService>), AppError> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    if input.amount < 0.0 {
        return Err(AppError::Validation("amount must be >= 0".into()));
    }
    let service =
        Booking::add_extra_service(&state.pool, id, &input.name, input.amount, user.id).await?;
    Ok((StatusCode::CREATED, Json(service)))
}

async fn remove_extra_service(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path((id, sid)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    if Booking::remove_extra_service(&state.pool, id, sid).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("extra service not found".into()))
    }
}

async fn income_summary(
    RequireAdmin(_user): RequireAdmin,
    State(state): State<AppState>,
    Query(params): Query<IncomeSummaryParams>,
) -> Result<Json<Vec<IncomeSummaryRow>>, AppError> {
    Ok(Json(
        Booking::income_summary(&state.pool, params.year, params.month).await?,
    ))
}

async fn room_availability(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Query(params): Query<AvailabilityParams>,
) -> Result<Json<Vec<RoomAvailability>>, AppError> {
    let (check_in, check_out) =
        parse_and_validate_dates(&params.check_in, &params.check_out, false)?;
    Ok(Json(
        Booking::room_availability(&state.pool, check_in, check_out).await?,
    ))
}

async fn today_summary(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<TodaySummary>, AppError> {
    Ok(Json(Booking::today_summary(&state.pool).await?))
}

const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024; // 10 MB
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "pdf", "webp", "gif"];

async fn upload_document(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(booking_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<BookingDocument>), AppError> {
    // Verify booking exists
    Booking::find_by_id(&state.pool, booking_id)
        .await?
        .ok_or_else(|| AppError::NotFound("booking not found".into()))?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(e.to_string()))?
        .ok_or_else(|| AppError::Validation("no file provided".into()))?;

    let original_filename = field
        .file_name()
        .unwrap_or("document")
        .to_string();
    let mime_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();
    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::Validation(e.to_string()))?;

    if data.is_empty() {
        return Err(AppError::Validation("file is empty".into()));
    }
    if data.len() > MAX_UPLOAD_BYTES {
        return Err(AppError::Validation("file too large (max 10 MB)".into()));
    }

    let ext = std::path::Path::new(&original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin")
        .to_lowercase();

    if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(AppError::Validation(
            "only JPG, PNG, PDF, WebP, GIF files are allowed".into(),
        ));
    }

    let stored_name = format!("{}.{}", Uuid::new_v4(), ext);
    let dir = state.uploads_dir.join(booking_id.to_string());
    tokio::fs::create_dir_all(&dir).await?;
    tokio::fs::write(dir.join(&stored_name), &data).await?;

    let doc = BookingDocument::create(
        &state.pool,
        booking_id,
        &original_filename,
        &stored_name,
        &mime_type,
        data.len() as i64,
        user.id,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(doc)))
}

async fn download_document(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path((booking_id, doc_id)): Path<(Uuid, Uuid)>,
) -> Result<Response<Body>, AppError> {
    let doc = BookingDocument::find_by_id(&state.pool, doc_id)
        .await?
        .filter(|d| d.booking_id == booking_id)
        .ok_or_else(|| AppError::NotFound("document not found".into()))?;

    let path = state
        .uploads_dir
        .join(booking_id.to_string())
        .join(&doc.stored_name);

    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| AppError::NotFound("file not found on disk".into()))?;

    let disposition = format!("inline; filename=\"{}\"", doc.filename);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &doc.mime_type)
        .header(header::CONTENT_DISPOSITION, disposition)
        .header(header::CONTENT_LENGTH, bytes.len())
        .body(Body::from(bytes))
        .unwrap())
}

async fn delete_document(
    AuthUser(_user): AuthUser,
    State(state): State<AppState>,
    Path((booking_id, doc_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let doc = BookingDocument::delete(&state.pool, doc_id)
        .await?
        .filter(|d| d.booking_id == booking_id)
        .ok_or_else(|| AppError::NotFound("document not found".into()))?;

    // Best-effort file removal — don't fail if file is already gone
    let path = state
        .uploads_dir
        .join(booking_id.to_string())
        .join(&doc.stored_name);
    let _ = tokio::fs::remove_file(&path).await;

    Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/income-summary", get(income_summary))
        .route("/room-availability", get(room_availability))
        .route("/today-summary", get(today_summary))
        .route("/", get(list_bookings).post(create_booking))
        .route("/{id}", get(get_booking).put(update_booking))
        .route("/{id}/cancel", put(cancel_booking))
        .route("/{id}/rooms/{room_id}/cancel", put(cancel_room))
        .route("/{id}/extra-services", post(add_extra_service))
        .route("/{id}/extra-services/{sid}", delete(remove_extra_service))
        .route("/{id}/documents", post(upload_document))
        .route("/{id}/documents/{doc_id}", get(download_document).delete(delete_document))
}
