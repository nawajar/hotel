<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import AppShell from "@/components/AppShell.vue";
import { useIncomeSummaryQuery } from "@/composables/useBookingsQueries";

const { t } = useI18n();

const summaryYear = ref(new Date().getFullYear());
const summaryMonth = ref<number | undefined>(undefined);
const { data: incomeSummary } = useIncomeSummaryQuery(summaryYear, summaryMonth);

const summaryMonthOptions = computed(() => [
  { value: undefined, label: t("adminBookings.monthAll") },
  ...Array.from({ length: 12 }, (_, i) => ({
    value: i + 1,
    label: String(i + 1).padStart(2, "0"),
  })),
]);

function formatPrice(price: number) {
  return price.toLocaleString();
}
</script>

<template>
  <AppShell>
    <div class="bg-white rounded-lg border border-gray-200 p-6">
      <h1 class="text-lg font-semibold text-gray-900 mb-4">
        {{ t("nav.incomeSummary") }}
      </h1>

      <div class="flex items-end gap-4 mb-6">
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-gray-700">{{ t("adminBookings.year") }}</label>
          <input
            v-model.number="summaryYear"
            type="number"
            :min="2000"
            :max="2100"
            class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400 w-28"
          />
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-xs font-medium text-gray-700">{{ t("adminBookings.month") }}</label>
          <select
            v-model="summaryMonth"
            class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400"
          >
            <option
              v-for="opt in summaryMonthOptions"
              :key="String(opt.value)"
              :value="opt.value"
            >
              {{ opt.label }}
            </option>
          </select>
        </div>
      </div>

      <table class="w-full text-sm">
        <thead>
          <tr class="text-left text-gray-500 border-b border-gray-200">
            <th class="py-2 pr-4 font-medium">{{ t("adminBookings.period") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminBookings.bookingCount") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminBookings.roomRevenue") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminBookings.extraRevenue") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminBookings.discountTotal") }}</th>
            <th class="py-2 font-medium">{{ t("adminBookings.netRevenue") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="!incomeSummary || incomeSummary.length === 0">
            <td colspan="6" class="py-8 text-center text-sm text-gray-400">
              {{ t("adminBookings.emptyState") }}
            </td>
          </tr>
          <tr
            v-for="row in incomeSummary"
            :key="row.period"
            class="border-b border-gray-100"
          >
            <td class="py-2 pr-4 font-mono text-gray-900">{{ row.period }}</td>
            <td class="py-2 pr-4 text-gray-600">{{ row.booking_count }}</td>
            <td class="py-2 pr-4 text-gray-600">₭{{ formatPrice(row.room_revenue) }}</td>
            <td class="py-2 pr-4 text-gray-600">₭{{ formatPrice(row.extra_revenue) }}</td>
            <td class="py-2 pr-4 text-gray-600">₭{{ formatPrice(row.discount_total) }}</td>
            <td class="py-2 text-gray-900 font-medium">₭{{ formatPrice(row.net_revenue) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </AppShell>
</template>
