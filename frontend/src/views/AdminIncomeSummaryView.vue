<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import AppShell from "@/components/AppShell.vue";
import { useIncomeSummaryQuery, useIncomeDetailQuery } from "@/composables/useBookingsQueries";
import { useSettingsStore } from "@/stores/settings";

const { t } = useI18n();
const settingsStore = useSettingsStore();

const summaryYear = ref(new Date().getFullYear());
const summaryMonth = ref<number | undefined>(undefined);
const monthSelected = computed(() => summaryMonth.value !== undefined);
const summaryMonthForDetail = computed(() => summaryMonth.value ?? 1);

const { data: incomeSummary } = useIncomeSummaryQuery(summaryYear, summaryMonth);
const { data: incomeDetail } = useIncomeDetailQuery(
  summaryYear,
  summaryMonthForDetail,
  monthSelected,
);

const summaryMonthOptions = computed(() => [
  { value: undefined, label: t("adminBookings.monthAll") },
  ...Array.from({ length: 12 }, (_, i) => ({
    value: i + 1,
    label: String(i + 1).padStart(2, "0"),
  })),
]);

async function downloadPdf() {
  const rows = incomeDetail.value;
  if (!rows || rows.length === 0) return;

  const { default: jsPDF } = await import("jspdf");
  const { default: autoTable } = await import("jspdf-autotable");

  const doc = new jsPDF({ orientation: "landscape" });

  const monthLabel = String(summaryMonth.value).padStart(2, "0");
  const title = `Income Summary — ${summaryYear.value}-${monthLabel}`;

  doc.setFontSize(14);
  doc.text(title, 14, 16);
  doc.setFontSize(9);
  doc.setTextColor(120);
  doc.text(new Date().toLocaleString(), 14, 22);
  doc.setTextColor(0);

  const totalRoom = rows.reduce((s, r) => s + r.room_revenue, 0);
  const totalExtra = rows.reduce((s, r) => s + r.extra_revenue, 0);
  const totalDiscount = rows.reduce((s, r) => s + r.discount_total, 0);
  const totalNet = rows.reduce((s, r) => s + r.net_revenue, 0);

  autoTable(doc, {
    startY: 28,
    head: [["Date Paid", "Booking Ref", "Customer", "Room Revenue", "Extra", "Discount", "Net Revenue"]],
    body: [
      ...rows.map((r) => [
        settingsStore.formatDate(r.paid_date),
        r.booking_ref,
        r.customer_name ?? "-",
        r.room_revenue.toLocaleString(),
        r.extra_revenue.toLocaleString(),
        r.discount_total.toLocaleString(),
        r.net_revenue.toLocaleString(),
      ]),
      [
        { content: `Total (${rows.length})`, colSpan: 3, styles: { fontStyle: "bold" } },
        { content: totalRoom.toLocaleString(), styles: { fontStyle: "bold" } },
        { content: totalExtra.toLocaleString(), styles: { fontStyle: "bold" } },
        { content: totalDiscount.toLocaleString(), styles: { fontStyle: "bold" } },
        { content: totalNet.toLocaleString(), styles: { fontStyle: "bold" } },
      ],
    ],
    styles: { fontSize: 8 },
    headStyles: { fillColor: [40, 40, 40] },
    foot: [],
  });

  doc.save(`income-${summaryYear.value}-${monthLabel}.pdf`);
}
</script>

<template>
  <AppShell>
    <div class="bg-white rounded-lg border border-gray-200 p-6">
      <h1 class="text-lg font-semibold text-gray-900 mb-4">
        {{ t("nav.incomeSummary") }}
      </h1>

      <div class="flex items-end justify-between gap-4 mb-6">
        <div class="flex items-end gap-4">
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
        <button
          v-if="monthSelected"
          class="flex items-center gap-2 rounded-md bg-gray-900 text-white px-4 py-2 text-sm font-medium hover:bg-gray-700 transition-colors"
          @click="downloadPdf"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          {{ t("adminBookings.downloadPdf") }}
        </button>
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
            <td class="py-2 pr-4 text-gray-600">{{ settingsStore.formatPrice(row.room_revenue) }}</td>
            <td class="py-2 pr-4 text-gray-600">{{ settingsStore.formatPrice(row.extra_revenue) }}</td>
            <td class="py-2 pr-4 text-gray-600">{{ settingsStore.formatPrice(row.discount_total) }}</td>
            <td class="py-2 text-gray-900 font-medium">{{ settingsStore.formatPrice(row.net_revenue) }}</td>
          </tr>
        </tbody>
      </table>

    </div>
  </AppShell>
</template>
