import { apiClient } from "./client";

export interface BookingRoom {
  id: string;
  booking_id: string;
  room_id: string;
  room_number: string;
  room_name: string;
  price_snapshot: number;
  status: "active" | "cancelled";
  updated_at: string;
  updated_by: string;
}

export interface BookingExtraService {
  id: string;
  booking_id: string;
  name: string;
  amount: number;
  created_at: string;
  updated_by: string;
}

export interface Booking {
  id: string;
  booking_ref: string;
  status: "active" | "cancelled";
  payment_status: "paid" | "unpaid";
  payment_method: "cash" | "bank_transfer" | null;
  check_in: string;
  check_out: string;
  note: string | null;
  discount_type: "amount" | "percentage" | null;
  discount_value: number | null;
  customer_name: string | null;
  customer_phone: string | null;
  customer_id_type: string | null;
  customer_id_number: string | null;
  created_at: string;
  created_by: string;
  created_by_name: string;
  updated_at: string;
  updated_by: string;
  updated_by_name: string;
  deposit_amount: number | null;
  deposit_returned: boolean;
  deposit_returned_at: string | null;
  deposit_returned_by: string | null;
  paid_at: string | null;
  actual_check_in: string | null;
  actual_check_out: string | null;
}

export interface BookingDocument {
  id: string;
  booking_id: string;
  filename: string;
  mime_type: string;
  size: number;
  uploaded_by: string;
  uploaded_by_name: string;
  created_at: string;
  category: string;
}

export interface BookingDetail {
  booking: Booking;
  rooms: BookingRoom[];
  extra_services: BookingExtraService[];
  documents: BookingDocument[];
}

export interface RoomAvailability {
  id: string;
  room_number: string;
  room_name: string;
  price: number;
  is_available: boolean;
}


export interface TodaySummary {
  total_rooms: number;
  available_now: number;
  occupied_now: number;
  check_ins_today: number;
  check_outs_today: number;
  needs_attention: number;
  unpaid_active: number;
}

export interface IncomeSummaryRow {
  period: string;
  booking_count: number;
  room_revenue: number;
  extra_revenue: number;
  discount_total: number;
  net_revenue: number;
}

export interface IncomeDetailRow {
  booking_ref: string;
  customer_name: string | null;
  paid_date: string;
  room_revenue: number;
  extra_revenue: number;
  discount_total: number;
  net_revenue: number;
}

export interface CreateBookingInput {
  check_in: string;
  check_out: string;
  room_ids: string[];
  note?: string;
  discount_type?: string;
  discount_value?: number;
  payment_status?: string;
  payment_method?: string;
  deposit_amount?: number;
  customer_name?: string;
  customer_phone?: string;
  customer_id_type?: string;
  customer_id_number?: string;
}

export interface UpdateBookingInput {
  check_in: string;
  check_out: string;
  note?: string;
  discount_type?: string;
  discount_value?: number;
  payment_status?: string;
  payment_method?: string;
  deposit_amount?: number;
  customer_name?: string;
  customer_phone?: string;
  customer_id_type?: string;
  customer_id_number?: string;
}

export interface DailyReportRow {
  date: string;
  booking_count: number;
  cash_revenue: number;
  bank_transfer_revenue: number;
  unspecified_revenue: number;
  total_revenue: number;
  cumulative_total: number;
  deposit_held_count: number;
  deposit_held_amount: number;
  unpaid_count: number;
  unpaid_amount: number;
}

export interface DepositSummary {
  count: number;
  total_amount: number;
}

export interface DailyReportDetailRow {
  record_type: "paid" | "unpaid";
  booking_ref: string;
  customer_name: string | null;
  check_in: string;
  check_out: string;
  payment_method: string | null;
  rooms: string;
  net_revenue: number;
  deposit_amount: number | null;
  deposit_returned: boolean;
}

export interface AddExtraServiceInput {
  name: string;
  amount: number;
}

export const bookingsApi = {
  listBookings: () => apiClient.get<Booking[]>("/bookings"),
  getBooking: (id: string) => apiClient.get<BookingDetail>(`/bookings/${id}`),
  createBooking: (input: CreateBookingInput) =>
    apiClient.post<BookingDetail>("/bookings", input),
  updateBooking: (id: string, input: UpdateBookingInput) =>
    apiClient.put<BookingDetail>(`/bookings/${id}`, input),
  cancelBooking: (id: string) => apiClient.put<void>(`/bookings/${id}/cancel`),
  cancelBookingRoom: (bookingId: string, roomBookingId: string) =>
    apiClient.put<BookingRoom>(`/bookings/${bookingId}/rooms/${roomBookingId}/cancel`),
  addExtraService: (bookingId: string, input: AddExtraServiceInput) =>
    apiClient.post<BookingExtraService>(`/bookings/${bookingId}/extra-services`, input),
  removeExtraService: (bookingId: string, serviceId: string) =>
    apiClient.del<void>(`/bookings/${bookingId}/extra-services/${serviceId}`),
  getIncomeSummary: (year: number, month?: number) => {
    const params = new URLSearchParams({ year: String(year) });
    if (month !== undefined) params.set("month", String(month));
    return apiClient.get<IncomeSummaryRow[]>(`/bookings/income-summary?${params}`);
  },
  getIncomeDetail: (year: number, month: number) => {
    const params = new URLSearchParams({ year: String(year), month: String(month) });
    return apiClient.get<IncomeDetailRow[]>(`/bookings/income-detail?${params}`);
  },
  getRoomAvailability: (check_in: string, check_out: string) => {
    const params = new URLSearchParams({ check_in, check_out });
    return apiClient.get<RoomAvailability[]>(`/bookings/room-availability?${params}`);
  },
  getTodaySummary: () =>
    apiClient.get<TodaySummary>(`/bookings/today-summary`),
  returnDeposit: (bookingId: string) =>
    apiClient.put<BookingDetail>(`/bookings/${bookingId}/return-deposit`),
  checkIn: (bookingId: string) =>
    apiClient.put<BookingDetail>(`/bookings/${bookingId}/check-in`),
  checkOut: (bookingId: string) =>
    apiClient.put<BookingDetail>(`/bookings/${bookingId}/check-out`),

  uploadDocument: async (bookingId: string, file: File, category: string = "general"): Promise<BookingDocument> => {
    const form = new FormData();
    form.append("category", category);
    form.append("file", file);
    const res = await fetch(`/api/bookings/${bookingId}/documents`, {
      method: "POST",
      credentials: "include",
      body: form,
    });
    if (!res.ok) {
      let message = res.statusText;
      try {
        const body = await res.json();
        if (body?.error) message = body.error;
      } catch { /* empty */ }
      const { ApiError } = await import("./client");
      throw new ApiError(res.status, message);
    }
    return res.json();
  },
  deleteDocument: (bookingId: string, docId: string) =>
    apiClient.del<void>(`/bookings/${bookingId}/documents/${docId}`),
  documentDownloadUrl: (bookingId: string, docId: string) =>
    `/api/bookings/${bookingId}/documents/${docId}`,
  getDailyReport: (year: number, month: number) => {
    const params = new URLSearchParams({ year: String(year), month: String(month) });
    return apiClient.get<DailyReportRow[]>(`/bookings/daily-report?${params}`);
  },
  getDepositSummary: () =>
    apiClient.get<DepositSummary>(`/bookings/deposit-summary`),
  getDailyReportDetail: (year: number, month: number, day: number) => {
    const params = new URLSearchParams({ year: String(year), month: String(month), day: String(day) });
    return apiClient.get<DailyReportDetailRow[]>(`/bookings/daily-report-detail?${params}`);
  },
};
