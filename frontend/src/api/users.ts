import { apiClient } from "./client";

export interface UserListRow {
  id: string;
  email: string;
  name: string;
  role: "admin" | "employee";
  created_at: string;
}

export interface CreateUserInput {
  email: string;
  name: string;
  password: string;
  role: "admin" | "employee";
}

export const usersApi = {
  list: () => apiClient.get<UserListRow[]>("/admin/users"),
  create: (input: CreateUserInput) => apiClient.post<UserListRow>("/admin/users", input),
};
