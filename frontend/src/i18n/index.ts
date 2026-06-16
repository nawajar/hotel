import { createI18n } from "vue-i18n";
import en from "@/locales/en.json";
import lo from "@/locales/lo.json";
import { unflattenMessages } from "./flatten";

export const SUPPORTED_LOCALES = ["en", "lo"] as const;
export type Locale = (typeof SUPPORTED_LOCALES)[number];

const LOCALE_STORAGE_KEY = "locale";

function getInitialLocale(): Locale {
  const stored = localStorage.getItem(LOCALE_STORAGE_KEY);
  return (SUPPORTED_LOCALES as readonly string[]).includes(stored ?? "")
    ? (stored as Locale)
    : "en";
}

export const i18n = createI18n({
  legacy: false,
  locale: getInitialLocale(),
  fallbackLocale: "en",
  messages: { en, lo },
});

export function setLocale(locale: Locale) {
  i18n.global.locale.value = locale;
  localStorage.setItem(LOCALE_STORAGE_KEY, locale);
}

export function getLocale(): Locale {
  return i18n.global.locale.value as Locale;
}

/** Merges admin-saved overrides (flat dotted keys per locale) on top of the bundled defaults. */
export function applyTranslationOverrides(overrides: Record<string, Record<string, string>>) {
  for (const [locale, flat] of Object.entries(overrides)) {
    if (!(SUPPORTED_LOCALES as readonly string[]).includes(locale)) continue;
    i18n.global.mergeLocaleMessage(locale, unflattenMessages(flat));
  }
}

/** Updates a single key's live message, so an admin edit applies without a page reload. */
export function setMessageLive(locale: Locale, key: string, value: string) {
  i18n.global.mergeLocaleMessage(locale, unflattenMessages({ [key]: value }));
}
