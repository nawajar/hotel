import { defineStore } from "pinia";
import { settingsApi } from "@/api/settings";

const DEFAULT_TIMEZONE = "Asia/Bangkok";
const DEFAULT_PRICE_SYMBOL = "₭";
const DEFAULT_DATE_FORMAT = "DD/MM/YYYY";
const DEFAULT_FONT_SIZE = "medium";

const FONT_SIZE_MAP: Record<string, string> = {
  small: "14px",
  medium: "16px",
  large: "18px",
};

export const useSettingsStore = defineStore("settings", {
  state: () => ({
    timezone: DEFAULT_TIMEZONE,
    priceSymbol: DEFAULT_PRICE_SYMBOL,
    dateFormat: DEFAULT_DATE_FORMAT,
    fontSize: DEFAULT_FONT_SIZE,
  }),
  actions: {
    async load() {
      try {
        const data = await settingsApi.get();
        this.timezone = data.timezone;
        this.priceSymbol = data.price_symbol;
        this.dateFormat = data.date_format;
        this.fontSize = data.font_size;
        this.applyFontSize();
      } catch {
        this.timezone = DEFAULT_TIMEZONE;
        this.priceSymbol = DEFAULT_PRICE_SYMBOL;
        this.dateFormat = DEFAULT_DATE_FORMAT;
        this.fontSize = DEFAULT_FONT_SIZE;
        this.applyFontSize();
      }
    },
    applyFontSize() {
      const px = FONT_SIZE_MAP[this.fontSize] ?? FONT_SIZE_MAP[DEFAULT_FONT_SIZE];
      document.documentElement.style.setProperty("--base-font-size", px);
    },
    formatDate(iso: string | null | undefined): string {
      if (!iso) return "—";
      try {
        const date = new Date(iso);
        const fmt = this.dateFormat;
        const tz = this.timezone;

        if (fmt === "locale") {
          return new Intl.DateTimeFormat(undefined, {
            timeZone: tz,
            day: "2-digit",
            month: "short",
            year: "numeric",
          }).format(date);
        }

        // Use formatToParts to get day/month/year components
        const parts = new Intl.DateTimeFormat(undefined, {
          timeZone: tz,
          day: "2-digit",
          month: "2-digit",
          year: "numeric",
        }).formatToParts(date);

        let day = "";
        let month = "";
        let year = "";
        for (const part of parts) {
          if (part.type === "day") day = part.value.padStart(2, "0");
          else if (part.type === "month") month = part.value.padStart(2, "0");
          else if (part.type === "year") year = part.value;
        }

        if (fmt === "DD/MM/YYYY") return `${day}/${month}/${year}`;
        if (fmt === "MM/DD/YYYY") return `${month}/${day}/${year}`;
        if (fmt === "YYYY-MM-DD") return `${year}-${month}-${day}`;

        // fallback
        return new Intl.DateTimeFormat(undefined, {
          timeZone: tz,
          day: "2-digit",
          month: "short",
          year: "numeric",
        }).format(date);
      } catch {
        return iso ?? "—";
      }
    },
    formatDateTime(iso: string | null | undefined): string {
      if (!iso) return "—";
      try {
        const date = new Date(iso);
        const fmt = this.dateFormat;
        const tz = this.timezone;

        // Get date portion
        let datePart: string;
        if (fmt === "locale") {
          datePart = new Intl.DateTimeFormat(undefined, {
            timeZone: tz,
            day: "2-digit",
            month: "short",
            year: "numeric",
          }).format(date);
        } else {
          const parts = new Intl.DateTimeFormat(undefined, {
            timeZone: tz,
            day: "2-digit",
            month: "2-digit",
            year: "numeric",
          }).formatToParts(date);

          let day = "";
          let month = "";
          let year = "";
          for (const part of parts) {
            if (part.type === "day") day = part.value.padStart(2, "0");
            else if (part.type === "month") month = part.value.padStart(2, "0");
            else if (part.type === "year") year = part.value;
          }

          if (fmt === "DD/MM/YYYY") datePart = `${day}/${month}/${year}`;
          else if (fmt === "MM/DD/YYYY") datePart = `${month}/${day}/${year}`;
          else if (fmt === "YYYY-MM-DD") datePart = `${year}-${month}-${day}`;
          else datePart = `${day}/${month}/${year}`;
        }

        // Get time portion (HH:MM)
        const timePart = new Intl.DateTimeFormat(undefined, {
          timeZone: tz,
          hour: "2-digit",
          minute: "2-digit",
        }).format(date);

        return `${datePart} ${timePart}`;
      } catch {
        return iso ?? "—";
      }
    },
    formatPrice(price: number): string {
      return `${this.priceSymbol}${price.toLocaleString()}`;
    },
  },
});
