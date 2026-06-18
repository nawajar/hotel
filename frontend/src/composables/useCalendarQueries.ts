import { computed, type Ref } from "vue";
import { useQuery, keepPreviousData } from "@tanstack/vue-query";
import { calendarApi } from "@/api/calendar";

export function useCalendarRoomsQuery() {
  return useQuery({
    queryKey: ["calendar-rooms"],
    queryFn: () => calendarApi.getRooms(),
    staleTime: 5 * 60 * 1000,
  });
}

export function useCalendarBookingsQuery(start: Ref<string>, end: Ref<string>) {
  return useQuery({
    queryKey: computed(() => ["calendar-bookings", start.value, end.value]),
    queryFn: () => calendarApi.getBookings(start.value, end.value),
    enabled: computed(() => !!start.value && !!end.value),
    staleTime: 60 * 1000,
    placeholderData: keepPreviousData,
  });
}
