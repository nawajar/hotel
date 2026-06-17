import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { roomsApi, type CreateRoomInput, type UpdateRoomInput } from "@/api/rooms";

export function useRoomsQuery() {
  return useQuery({
    queryKey: ["admin-rooms"],
    queryFn: roomsApi.list,
  });
}

export function useCreateRoomMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateRoomInput) => roomsApi.create(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin-rooms"] });
    },
  });
}

export function useUpdateRoomMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: UpdateRoomInput }) =>
      roomsApi.update(id, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin-rooms"] });
    },
  });
}

export function useToggleRoomMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => roomsApi.toggle(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin-rooms"] });
    },
  });
}

export function useDeleteRoomMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => roomsApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin-rooms"] });
    },
  });
}
