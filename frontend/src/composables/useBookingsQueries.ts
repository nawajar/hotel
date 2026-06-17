import { computed, type Ref } from "vue";
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import {
  bookingsApi,
  type CreateBookingInput,
  type UpdateBookingInput,
  type AddExtraServiceInput,
} from "@/api/bookings";

export function useUploadDocumentMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ bookingId, file }: { bookingId: string; file: File }) =>
      bookingsApi.uploadDocument(bookingId, file),
    onSuccess: (_data, { bookingId }) => {
      queryClient.invalidateQueries({ queryKey: ["admin-booking", bookingId] });
    },
  });
}

export function useDeleteDocumentMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ bookingId, docId }: { bookingId: string; docId: string }) =>
      bookingsApi.deleteDocument(bookingId, docId),
    onSuccess: (_data, { bookingId }) => {
      queryClient.invalidateQueries({ queryKey: ["admin-booking", bookingId] });
    },
  });
}

export function useTodaySummaryQuery() {
  return useQuery({
    queryKey: ["today-summary"],
    queryFn: bookingsApi.getTodaySummary,
  });
}

export function useRoomAvailabilityQuery(checkIn: Ref<string>, checkOut: Ref<string>) {
  return useQuery({
    queryKey: computed(() => ["room-availability", checkIn.value, checkOut.value]),
    queryFn: () => bookingsApi.getRoomAvailability(checkIn.value, checkOut.value),
    enabled: computed(() => !!checkIn.value && !!checkOut.value),
  });
}

export function useBookingsQuery() {
  return useQuery({
    queryKey: ["admin-bookings"],
    queryFn: bookingsApi.listBookings,
  });
}

export function useBookingQuery(id: Ref<string>) {
  return useQuery({
    queryKey: computed(() => ["admin-booking", id.value]),
    queryFn: () => bookingsApi.getBooking(id.value),
    enabled: computed(() => !!id.value),
  });
}

export function useCreateBookingMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateBookingInput) => bookingsApi.createBooking(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin-bookings"] });
      queryClient.invalidateQueries({ queryKey: ["today-summary"] });
    },
  });
}

export function useUpdateBookingMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: UpdateBookingInput }) =>
      bookingsApi.updateBooking(id, input),
    onSuccess: (_data, { id }) => {
      queryClient.invalidateQueries({ queryKey: ["admin-bookings"] });
      queryClient.invalidateQueries({ queryKey: ["admin-booking", id] });
    },
  });
}

export function useCancelBookingMutation(id: Ref<string>) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => bookingsApi.cancelBooking(id.value),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin-bookings"] });
      queryClient.invalidateQueries({ queryKey: ["admin-booking", id.value] });
      queryClient.invalidateQueries({ queryKey: ["today-summary"] });
    },
  });
}

export function useCancelBookingRoomMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      bookingId,
      roomBookingId,
    }: {
      bookingId: string;
      roomBookingId: string;
    }) => bookingsApi.cancelBookingRoom(bookingId, roomBookingId),
    onSuccess: (_data, { bookingId }) => {
      queryClient.invalidateQueries({ queryKey: ["admin-booking", bookingId] });
      queryClient.invalidateQueries({ queryKey: ["admin-bookings"] });
    },
  });
}

export function useAddExtraServiceMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      bookingId,
      input,
    }: {
      bookingId: string;
      input: AddExtraServiceInput;
    }) => bookingsApi.addExtraService(bookingId, input),
    onSuccess: (_data, { bookingId }) => {
      queryClient.invalidateQueries({ queryKey: ["admin-booking", bookingId] });
    },
  });
}

export function useRemoveExtraServiceMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      bookingId,
      serviceId,
    }: {
      bookingId: string;
      serviceId: string;
    }) => bookingsApi.removeExtraService(bookingId, serviceId),
    onSuccess: (_data, { bookingId }) => {
      queryClient.invalidateQueries({ queryKey: ["admin-booking", bookingId] });
    },
  });
}

export function useIncomeSummaryQuery(
  year: Ref<number>,
  month: Ref<number | undefined>,
) {
  return useQuery({
    queryKey: computed(() => ["income-summary", year.value, month.value]),
    queryFn: () => bookingsApi.getIncomeSummary(year.value, month.value),
  });
}
