import { defineStore } from "pinia";
import { settingsApi } from "@/api/settings";

const DEFAULT_TIMEZONE = "Asia/Bangkok";
const DEFAULT_PRICE_SYMBOL = "₭";

export const useSettingsStore = defineStore("settings", {
  state: () => ({
    timezone: DEFAULT_TIMEZONE,
    priceSymbol: DEFAULT_PRICE_SYMBOL,
  }),
  actions: {
    async load() {
      try {
        const data = await settingsApi.get();
        this.timezone = data.timezone;
        this.priceSymbol = data.price_symbol;
      } catch {
        this.timezone = DEFAULT_TIMEZONE;
        this.priceSymbol = DEFAULT_PRICE_SYMBOL;
      }
    },
    formatDate(iso: string): string {
      const opts = { day: "2-digit", month: "short", year: "numeric" } as const;
      try {
        return new Intl.DateTimeFormat(undefined, { timeZone: this.timezone, ...opts }).format(new Date(iso));
      } catch {
        try {
          return new Intl.DateTimeFormat(undefined, { timeZone: DEFAULT_TIMEZONE, ...opts }).format(new Date(iso));
        } catch {
          return iso;
        }
      }
    },
    formatPrice(price: number): string {
      return `${this.priceSymbol}${price.toLocaleString()}`;
    },
  },
});
