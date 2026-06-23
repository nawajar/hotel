import { apiClient } from "./client";

export interface SystemSettings {
  timezone: string;
  price_symbol: string;
  date_format: string;
  font_size: string;
}

export interface UpdateSettingsInput {
  timezone?: string;
  price_symbol?: string;
  date_format?: string;
  font_size?: string;
}

export const settingsApi = {
  get: () => apiClient.get<SystemSettings>("/admin/settings"),
  update: (input: UpdateSettingsInput) =>
    apiClient.put<SystemSettings>("/admin/settings", input),
};
