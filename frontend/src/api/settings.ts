import { apiClient } from "./client";

export interface SystemSettings {
  timezone: string;
  price_symbol: string;
}

export interface UpdateSettingsInput {
  timezone?: string;
  price_symbol?: string;
}

export const settingsApi = {
  get: () => apiClient.get<SystemSettings>("/admin/settings"),
  update: (input: UpdateSettingsInput) =>
    apiClient.put<SystemSettings>("/admin/settings", input),
};
