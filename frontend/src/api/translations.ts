import { apiClient } from "./client";

export interface TranslationOverride {
  key: string;
  locale: string;
  value: string;
}

export const translationsApi = {
  publicOverrides: () => apiClient.get<Record<string, Record<string, string>>>("/translations"),
  adminList: () => apiClient.get<TranslationOverride[]>("/admin/translations"),
  adminUpsert: (key: string, locale: string, value: string) =>
    apiClient.put<void>("/admin/translations", { key, locale, value }),
  adminDelete: (key: string, locale: string) =>
    apiClient.del<void>("/admin/translations", { key, locale }),
};
