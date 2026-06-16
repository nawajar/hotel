import { apiClient } from "./client";

export interface UserPublic {
  id: string;
  email: string;
  name: string;
  role: "admin" | "employee";
}

export interface LoginInput {
  email: string;
  password: string;
}

export const authApi = {
  me: () => apiClient.get<UserPublic>("/auth/me"),
  login: (input: LoginInput) => apiClient.post<UserPublic>("/auth/login", input),
  logout: () => apiClient.post<void>("/auth/logout"),
  adminPing: () => apiClient.get<{ ok: boolean }>("/admin/ping"),
};
