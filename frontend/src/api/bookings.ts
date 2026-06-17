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
  check_in: string;
  check_out: string;
  label: "check_in" | "check_out" | "needs_attention" | null;
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

export interface CreateBookingInput {
  check_in: string;
  check_out: string;
  room_ids: string[];
  label?: string;
  note?: string;
  discount_type?: string;
  discount_value?: number;
  payment_status?: string;
  customer_name?: string;
  customer_phone?: string;
  customer_id_type?: string;
  customer_id_number?: string;
}

export interface UpdateBookingInput {
  check_in: string;
  check_out: string;
  label?: string;
  note?: string;
  discount_type?: string;
  discount_value?: number;
  payment_status?: string;
  customer_name?: string;
  customer_phone?: string;
  customer_id_type?: string;
  customer_id_number?: string;
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
  getRoomAvailability: (check_in: string, check_out: string) => {
    const params = new URLSearchParams({ check_in, check_out });
    return apiClient.get<RoomAvailability[]>(`/bookings/room-availability?${params}`);
  },
  getTodaySummary: () => apiClient.get<TodaySummary>("/bookings/today-summary"),
  uploadDocument: async (bookingId: string, file: File): Promise<BookingDocument> => {
    const form = new FormData();
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
};
