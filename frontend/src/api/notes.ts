import { apiClient } from "./client";

export interface Note {
  id: string;
  user_id: string;
  body: string;
  done: boolean;
  created_by_name: string;
  created_at: string;
  updated_at: string;
}

export const notesApi = {
  list: () => apiClient.get<Note[]>("/notes"),
  create: (body: string) => apiClient.post<void>("/notes", { body }),
  update: (id: string, done: boolean) => apiClient.patch<void>(`/notes/${id}`, { done }),
  delete: (id: string) => apiClient.del<void>(`/notes/${id}`),
};
