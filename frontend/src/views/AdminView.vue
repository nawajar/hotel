<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { useI18n } from "vue-i18n";
import AppShell from "@/components/AppShell.vue";
import { authApi } from "@/api/auth";
import { ApiError } from "@/api/client";
import { settingsApi } from "@/api/settings";
import { useSettingsStore } from "@/stores/settings";

const TIMEZONES: { label: string; value: string }[] = [
  { label: "UTC+07:00 — Bangkok, Hanoi, Jakarta", value: "Asia/Bangkok" },
  { label: "UTC+07:00 — Ho Chi Minh City", value: "Asia/Ho_Chi_Minh" },
  { label: "UTC+07:00 — Vientiane", value: "Asia/Vientiane" },
  { label: "UTC+07:00 — Phnom Penh", value: "Asia/Phnom_Penh" },
  { label: "UTC+06:30 — Yangon (Rangoon)", value: "Asia/Rangoon" },
  { label: "UTC+06:00 — Dhaka", value: "Asia/Dhaka" },
  { label: "UTC+05:30 — Mumbai, New Delhi", value: "Asia/Kolkata" },
  { label: "UTC+05:45 — Kathmandu", value: "Asia/Kathmandu" },
  { label: "UTC+08:00 — Kuala Lumpur, Singapore", value: "Asia/Singapore" },
  { label: "UTC+08:00 — Beijing, Shanghai", value: "Asia/Shanghai" },
  { label: "UTC+08:00 — Manila", value: "Asia/Manila" },
  { label: "UTC+08:00 — Taipei", value: "Asia/Taipei" },
  { label: "UTC+09:00 — Tokyo, Osaka", value: "Asia/Tokyo" },
  { label: "UTC+09:00 — Seoul", value: "Asia/Seoul" },
  { label: "UTC+09:30 — Darwin", value: "Australia/Darwin" },
  { label: "UTC+10:00 — Sydney, Melbourne", value: "Australia/Sydney" },
  { label: "UTC+05:00 — Karachi", value: "Asia/Karachi" },
  { label: "UTC+04:00 — Dubai", value: "Asia/Dubai" },
  { label: "UTC+03:00 — Moscow", value: "Europe/Moscow" },
  { label: "UTC+02:00 — Helsinki, Riga", value: "Europe/Helsinki" },
  { label: "UTC+01:00 — Paris, Berlin, Amsterdam", value: "Europe/Paris" },
  { label: "UTC+00:00 — London (GMT)", value: "Europe/London" },
  { label: "UTC+00:00 — UTC", value: "UTC" },
  { label: "UTC−05:00 — New York, Toronto", value: "America/New_York" },
  { label: "UTC−06:00 — Chicago", value: "America/Chicago" },
  { label: "UTC−07:00 — Denver", value: "America/Denver" },
  { label: "UTC−08:00 — Los Angeles, Vancouver", value: "America/Los_Angeles" },
];

const { t } = useI18n();
const pingResult = ref("");
const settingsStore = useSettingsStore();

const DATE_FORMATS = computed(() => [
  { label: "DD/MM/YYYY", value: "DD/MM/YYYY" },
  { label: "MM/DD/YYYY", value: "MM/DD/YYYY" },
  { label: "YYYY-MM-DD", value: "YYYY-MM-DD" },
  { label: t("admin.settingsDateFormatLocale"), value: "locale" },
]);

const FONT_SIZES = computed(() => [
  { label: t("admin.settingsFontSizeSmall"), value: "small" },
  { label: t("admin.settingsFontSizeMedium"), value: "medium" },
  { label: t("admin.settingsFontSizeLarge"), value: "large" },
]);

const NUMBER_FORMATS = computed(() => [
  { label: "1,234.56", value: "1,234.56" },
  { label: "1.234,56", value: "1.234,56" },
]);

async function handlePing() {
  pingResult.value = "";
  try {
    const res = await authApi.adminPing();
    pingResult.value = t("admin.pingOk", { ok: res.ok });
  } catch (err) {
    pingResult.value =
      err instanceof ApiError ? t("admin.pingError", { status: err.status, message: err.message }) : "error";
  }
}

function isValidTimezone(tz: string): boolean {
  try { new Intl.DateTimeFormat(undefined, { timeZone: tz }); return true; } catch { return false; }
}

const selectedTimezone = ref(settingsStore.timezone);
watch(() => settingsStore.timezone, (tz) => { selectedTimezone.value = tz; }, { immediate: true });
watch(selectedTimezone, (tz) => { settingsForm.value.timezone = tz; });

const settingsForm = ref({
  timezone: settingsStore.timezone,
  priceSymbol: settingsStore.priceSymbol,
  dateFormat: settingsStore.dateFormat,
  fontSize: settingsStore.fontSize,
  numberFormat: settingsStore.numberFormat,
});
const settingsSaving = ref(false);
const settingsSaved = ref(false);
const settingsError = ref("");

watch(
  () => [settingsStore.timezone, settingsStore.priceSymbol, settingsStore.dateFormat, settingsStore.fontSize, settingsStore.numberFormat] as const,
  ([tz, sym, df, fs, nf]) => {
    settingsForm.value = { timezone: tz, priceSymbol: sym, dateFormat: df, fontSize: fs, numberFormat: nf };
  },
  { immediate: true },
);

async function handleSaveSettings() {
  settingsSaving.value = true;
  settingsSaved.value = false;
  settingsError.value = "";
  const tz = settingsForm.value.timezone.trim();
  if (tz && !isValidTimezone(tz)) {
    settingsError.value = t("admin.settingsInvalidTimezone");
    settingsSaving.value = false;
    return;
  }
  try {
    await settingsApi.update({
      timezone: settingsForm.value.timezone || undefined,
      price_symbol: settingsForm.value.priceSymbol || undefined,
      date_format: settingsForm.value.dateFormat || undefined,
      font_size: settingsForm.value.fontSize || undefined,
      number_format: settingsForm.value.numberFormat || undefined,
    });
    await settingsStore.load();
    settingsForm.value.timezone = settingsStore.timezone;
    settingsForm.value.priceSymbol = settingsStore.priceSymbol;
    settingsForm.value.dateFormat = settingsStore.dateFormat;
    settingsForm.value.fontSize = settingsStore.fontSize;
    settingsForm.value.numberFormat = settingsStore.numberFormat;
    settingsStore.applyFontSize();
    settingsSaved.value = true;
    setTimeout(() => { settingsSaved.value = false; }, 2000);
  } catch (err) {
    settingsError.value =
      err instanceof ApiError ? err.message : t("admin.settingsError");
  } finally {
    settingsSaving.value = false;
  }
}
</script>

<template>
  <AppShell>
    <div class="bg-white rounded-lg border border-gray-200 p-6 mb-6">
      <h1 class="text-lg font-semibold text-gray-900">{{ t("admin.title") }}</h1>
      <p class="mt-2 text-sm text-gray-600">{{ t("admin.description") }}</p>

      <button class="btn btn-sm mt-4 bg-gray-900 text-white hover:bg-gray-700 border-none" @click="handlePing">
        {{ t("admin.pingButton") }}
      </button>
      <p v-if="pingResult" class="mt-3 text-sm text-gray-700">{{ pingResult }}</p>
    </div>

    <!-- General Settings -->
    <div class="bg-white rounded-lg border border-gray-200 p-6">
      <h2 class="text-base font-semibold text-gray-900 mb-4">{{ t("admin.settingsTitle") }}</h2>
      <div class="flex flex-col gap-4 max-w-sm">
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-gray-700">{{ t("admin.settingsTimezone") }}</label>
          <select
            v-model="selectedTimezone"
            class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400 bg-white"
          >
            <option v-for="tz in TIMEZONES" :key="tz.value" :value="tz.value">{{ tz.label }}</option>
          </select>
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-gray-700">{{ t("admin.settingsPriceSymbol") }}</label>
          <input
            v-model="settingsForm.priceSymbol"
            type="text"
            class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400"
            placeholder="₭"
          />
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-gray-700">{{ t("admin.settingsDateFormat") }}</label>
          <select
            v-model="settingsForm.dateFormat"
            class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400 bg-white"
          >
            <option v-for="df in DATE_FORMATS" :key="df.value" :value="df.value">{{ df.label }}</option>
          </select>
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-gray-700">{{ t("admin.settingsFontSize") }}</label>
          <select
            v-model="settingsForm.fontSize"
            class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400 bg-white"
          >
            <option v-for="fs in FONT_SIZES" :key="fs.value" :value="fs.value">{{ fs.label }}</option>
          </select>
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-gray-700">{{ t("admin.settingsNumberFormat") }}</label>
          <select
            v-model="settingsForm.numberFormat"
            class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400 bg-white"
          >
            <option v-for="nf in NUMBER_FORMATS" :key="nf.value" :value="nf.value">{{ nf.label }}</option>
          </select>
        </div>
        <div class="flex items-center gap-3">
          <button
            class="btn btn-sm bg-gray-900 text-white hover:bg-gray-700 border-none"
            :disabled="settingsSaving"
            @click="handleSaveSettings"
          >
            {{ settingsSaving ? "…" : t("admin.settingsSave") }}
          </button>
          <span v-if="settingsSaved" class="text-sm text-green-600">{{ t("admin.settingsSaved") }}</span>
          <span v-if="settingsError" class="text-sm text-red-600">{{ settingsError }}</span>
        </div>
      </div>
    </div>
  </AppShell>
</template>
