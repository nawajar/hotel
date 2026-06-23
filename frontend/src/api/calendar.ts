import { apiClient } from "./client";

export interface CalendarRoom {
  id: string;
  room_number: string;
  room_name: string;
}

export interface CalendarBooking {
  id: string;
  booking_ref: string;
  status: "active" | "cancelled";
  payment_status: "paid" | "unpaid";
  check_in: string;
  check_out: string;
  customer_name: string | null;
  note: string | null;
  rooms: CalendarRoom[];
}

export const calendarApi = {
  getBookings: (start: string, end: string) => {
    const params = new URLSearchParams({ start, end });
    return apiClient.get<CalendarBooking[]>(`/calendar/bookings?${params}`);
  },
  getRooms: () => apiClient.get<CalendarRoom[]>("/calendar/rooms"),
};
