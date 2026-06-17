import { apiClient } from "./client";

export interface Room {
  id: string;
  room_number: string;
  room_name: string;
  room_description: string | null;
  is_active: boolean;
  price: number;
  updated_at: string;
  updated_by: string;
}

export interface CreateRoomInput {
  room_number: string;
  room_name: string;
  room_description: string | null;
  is_active: boolean;
  price: number;
}

export type UpdateRoomInput = CreateRoomInput;

export const roomsApi = {
  list: () => apiClient.get<Room[]>("/admin/rooms"),
  create: (input: CreateRoomInput) => apiClient.post<Room>("/admin/rooms", input),
  update: (id: string, input: UpdateRoomInput) =>
    apiClient.put<Room>(`/admin/rooms/${id}`, input),
  toggle: (id: string) => apiClient.put<Room>(`/admin/rooms/${id}/toggle`),
  delete: (id: string) => apiClient.del<void>(`/admin/rooms/${id}`),
};
