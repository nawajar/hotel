<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import AppShell from "@/components/AppShell.vue";
import { useDailyReportQuery } from "@/composables/useBookingsQueries";
import { bookingsApi, type DailyReportDetailRow } from "@/api/bookings";
import { useQuery } from "@tanstack/vue-query";
import { registerPdfFont } from "@/utils/pdfFont";
import { useSettingsStore } from "@/stores/settings";

const { t, locale } = useI18n();
const settingsStore = useSettingsStore();

const reportYear = ref(new Date().getFullYear());
const reportMonth = ref(new Date().getMonth() + 1);

const { data: reportRows } = useDailyReportQuery(reportYear, reportMonth);
const { data: depositSummary } = useQuery({
  queryKey: ["deposit-summary"],
  queryFn: () => bookingsApi.getDepositSummary(),
  refetchInterval: 30_000,
});

const monthOptions = Array.from({ length: 12 }, (_, i) => ({
  value: i + 1,
  label: String(i + 1).padStart(2, "0"),
}));

const totals = computed(() => {
  const rows = reportRows.value ?? [];
  return {
    booking_count: rows.reduce((s, r) => s + r.booking_count, 0),
    cash_revenue: rows.reduce((s, r) => s + r.cash_revenue, 0),
    bank_transfer_revenue: rows.reduce((s, r) => s + r.bank_transfer_revenue, 0),
    unspecified_revenue: rows.reduce((s, r) => s + r.unspecified_revenue, 0),
    total_revenue: rows.reduce((s, r) => s + r.total_revenue, 0),
    deposit_held_count: rows.reduce((s, r) => s + r.deposit_held_count, 0),
    deposit_held_amount: rows.reduce((s, r) => s + r.deposit_held_amount, 0),
    unpaid_count: rows.reduce((s, r) => s + r.unpaid_count, 0),
    unpaid_amount: rows.reduce((s, r) => s + r.unpaid_amount, 0),
  };
});

// --- expand / collapse ---
const expandedDates = ref<Set<string>>(new Set());
const detailCache = ref<Map<string, DailyReportDetailRow[]>>(new Map());
const loadingDates = ref<Set<string>>(new Set());

async function toggleExpand(date: string) {
  if (expandedDates.value.has(date)) {
    expandedDates.value.delete(date);
    expandedDates.value = new Set(expandedDates.value); // trigger reactivity
    return;
  }
  expandedDates.value = new Set([...expandedDates.value, date]);
  if (!detailCache.value.has(date)) {
    loadingDates.value = new Set([...loadingDates.value, date]);
    try {
      const [y, m, d] = date.split("-").map(Number);
      const rows = await bookingsApi.getDailyReportDetail(y, m, d);
      detailCache.value = new Map(detailCache.value).set(date, rows);
    } finally {
      loadingDates.value.delete(date);
      loadingDates.value = new Set(loadingDates.value);
    }
  }
}

function paymentBadgeClass(method: string | null) {
  if (method === "cash") return "badge-success";
  if (method === "bank_transfer") return "badge-info";
  return "badge-ghost";
}

function paymentLabel(method: string | null) {
  if (method === "cash") return t("adminBookings.paymentMethodCash");
  if (method === "bank_transfer") return t("adminBookings.paymentMethodBankTransfer");
  return "—";
}

async function downloadDayPdf(date: string, rows: DailyReportDetailRow[]) {
  const { default: jsPDF } = await import("jspdf");
  const { default: autoTable } = await import("jspdf-autotable");

  const doc = new jsPDF({ orientation: "landscape" });
  const fontName = await registerPdfFont(doc, locale.value);

  doc.setFont(fontName);
  doc.setFontSize(14);
  doc.text(t("dailyReport.pdfDayTitle", { date }), 14, 16);
  doc.setFontSize(9);
  doc.setTextColor(120);
  doc.text(t("dailyReport.pdfGenerated", { time: settingsStore.formatDateTime(new Date().toISOString()) }), 14, 22);
  doc.setTextColor(0);

  const paidRows = rows.filter(r => r.record_type === "paid");
  const unpaidRows = rows.filter(r => r.record_type === "unpaid");

  const cashTotal = paidRows.filter(r => r.payment_method === "cash").reduce((s, r) => s + r.net_revenue, 0);
  const bankTotal = paidRows.filter(r => r.payment_method === "bank_transfer").reduce((s, r) => s + r.net_revenue, 0);
  const unspecTotal = paidRows.filter(r => !r.payment_method).reduce((s, r) => s + r.net_revenue, 0);
  const paidTotal = paidRows.reduce((s, r) => s + r.net_revenue, 0);
  const unpaidTotal = unpaidRows.reduce((s, r) => s + r.net_revenue, 0);
  const grandTotal = paidTotal + unpaidTotal;

  const depositHeld = rows.filter(r => r.deposit_amount && !r.deposit_returned);
  const depositHeldTotal = depositHeld.reduce((s, r) => s + (r.deposit_amount ?? 0), 0);
  const depositReturnedCount = rows.filter(r => r.deposit_amount && r.deposit_returned).length;

  let paidRunning = 0;

  autoTable(doc, {
    startY: 28,
    head: [[
      t("dailyReport.pdfColNum"),
      t("dailyReport.detailRef"),
      t("dailyReport.detailGuest"),
      t("adminBookings.rooms"),
      t("adminBookings.checkIn"),
      t("adminBookings.checkOut"),
      t("dailyReport.detailPayment"),
      t("adminBookings.status"),
      t("dailyReport.detailAmount"),
      t("dailyReport.pdfColPaidRunning"),
      t("dailyReport.detailDeposit"),
    ]],
    body: [
      ...rows.map((r, i) => {
        if (r.record_type === "paid") paidRunning += r.net_revenue;
        const depositCell = r.deposit_amount
          ? `${settingsStore.priceSymbol}${r.deposit_amount.toLocaleString()} (${r.deposit_returned ? t("dailyReport.pdfDepositReturned") : t("dailyReport.depositHeld")})`
          : "—";
        return [
          String(i + 1),
          r.booking_ref,
          r.customer_name ?? "—",
          r.rooms || "—",
          settingsStore.formatDate(r.check_in),
          settingsStore.formatDate(r.check_out),
          r.payment_method === "cash"
            ? t("adminBookings.paymentMethodCash")
            : r.payment_method === "bank_transfer"
              ? t("adminBookings.paymentMethodBankTransfer")
              : "—",
          r.record_type === "paid" ? t("adminBookings.paymentPaid") : t("adminBookings.paymentUnpaid"),
          r.net_revenue.toLocaleString(),
          r.record_type === "paid" ? paidRunning.toLocaleString() : "—",
          depositCell,
        ];
      }),
      [
        { content: t("dailyReport.pdfRowSummary", { paid: paidRows.length, unpaid: unpaidRows.length, total: rows.length }), colSpan: 8, styles: { fontStyle: "bold", halign: "right" } },
        { content: grandTotal.toLocaleString(), styles: { fontStyle: "bold" } },
        { content: "", styles: { fontStyle: "bold" } },
        { content: depositHeld.length > 0 ? `${depositHeld.length} ${t("dailyReport.depositHeld")}` : "", styles: { fontStyle: "bold" } },
      ],
    ],
    styles: { fontSize: 8, font: fontName },
    headStyles: { fillColor: [40, 40, 40], font: fontName },
    columnStyles: { 8: { halign: "right" }, 9: { halign: "right" }, 10: { halign: "right" } },
  });

  const finalY = (doc as any).lastAutoTable.finalY + 8;
  doc.setFontSize(9);
  doc.setFont(fontName, "bold");
  doc.text(t("dailyReport.pdfPaidSummary"), 14, finalY);
  doc.setFont(fontName, "normal");
  doc.text(`${t("dailyReport.cashRevenue")}: ${settingsStore.priceSymbol}${cashTotal.toLocaleString()}`, 14, finalY + 6);
  doc.text(`${t("dailyReport.bankTransferRevenue")}: ${settingsStore.priceSymbol}${bankTotal.toLocaleString()}`, 14, finalY + 12);
  if (unspecTotal > 0) doc.text(`${t("dailyReport.unspecifiedRevenue")}: ${settingsStore.priceSymbol}${unspecTotal.toLocaleString()}`, 14, finalY + 18);
  const afterPayment = finalY + (unspecTotal > 0 ? 26 : 20);
  doc.setFont(fontName, "bold");
  doc.text(t("dailyReport.pdfTotalPaid", { amount: paidTotal.toLocaleString() }), 14, afterPayment);
  doc.setFont(fontName, "normal");
  doc.text(t("dailyReport.pdfTotalUnpaid", { amount: unpaidTotal.toLocaleString(), n: unpaidRows.length }), 14, afterPayment + 6);
  doc.setFont(fontName, "bold");
  doc.text(t("dailyReport.pdfGrandTotal", { amount: grandTotal.toLocaleString() }), 14, afterPayment + 14);

  doc.setFont(fontName, "bold");
  doc.text(t("dailyReport.pdfSecurityDeposits"), 14, afterPayment + 24);
  doc.setFont(fontName, "normal");
  doc.text(t("dailyReport.pdfDepositsHeld", { n: depositHeld.length, amount: depositHeldTotal.toLocaleString() }), 14, afterPayment + 30);
  doc.text(t("dailyReport.pdfDepositsReturned", { n: depositReturnedCount }), 14, afterPayment + 36);

  doc.save(`daily-report-${date}.pdf`);
}

async function downloadPdf() {
  const rows = reportRows.value;
  if (!rows || rows.length === 0) return;

  const { default: jsPDF } = await import("jspdf");
  const { default: autoTable } = await import("jspdf-autotable");

  const doc = new jsPDF({ orientation: "landscape" });
  const fontName = await registerPdfFont(doc, locale.value);

  const monthLabel = String(reportMonth.value).padStart(2, "0");

  doc.setFont(fontName);
  doc.setFontSize(14);
  doc.text(t("dailyReport.pdfMonthTitle", { year: reportYear.value, month: monthLabel }), 14, 16);
  doc.setFontSize(9);
  doc.setTextColor(120);
  doc.text(settingsStore.formatDateTime(new Date().toISOString()), 14, 22);
  doc.setTextColor(0);

  const t_totals = totals.value;

  autoTable(doc, {
    startY: 28,
    head: [[
      t("dailyReport.date"),
      t("adminBookings.paymentPaid"),
      t("dailyReport.cashRevenue"),
      t("dailyReport.bankTransferRevenue"),
      t("dailyReport.unspecifiedRevenue"),
      t("dailyReport.dayTotal"),
      t("dailyReport.runningTotal"),
      t("dailyReport.depositsHeldHeader"),
      t("dailyReport.pdfColDepositAmt"),
      t("dailyReport.unpaidCount"),
      t("dailyReport.unpaidAmount"),
    ]],
    body: [
      ...rows.map((r) => [
        settingsStore.formatDate(r.date),
        String(r.booking_count),
        r.cash_revenue.toLocaleString(),
        r.bank_transfer_revenue.toLocaleString(),
        r.unspecified_revenue.toLocaleString(),
        r.total_revenue.toLocaleString(),
        r.cumulative_total.toLocaleString(),
        r.deposit_held_count > 0 ? String(r.deposit_held_count) : "—",
        r.deposit_held_amount > 0 ? r.deposit_held_amount.toLocaleString() : "—",
        r.unpaid_count > 0 ? String(r.unpaid_count) : "—",
        r.unpaid_amount > 0 ? r.unpaid_amount.toLocaleString() : "—",
      ]),
      [
        { content: t("dailyReport.pdfTotalDays", { n: rows.length }), colSpan: 1, styles: { fontStyle: "bold" } },
        { content: String(t_totals.booking_count), styles: { fontStyle: "bold" } },
        { content: t_totals.cash_revenue.toLocaleString(), styles: { fontStyle: "bold" } },
        { content: t_totals.bank_transfer_revenue.toLocaleString(), styles: { fontStyle: "bold" } },
        { content: t_totals.unspecified_revenue.toLocaleString(), styles: { fontStyle: "bold" } },
        { content: t_totals.total_revenue.toLocaleString(), styles: { fontStyle: "bold" } },
        { content: "", styles: { fontStyle: "bold" } },
        { content: t_totals.deposit_held_count > 0 ? String(t_totals.deposit_held_count) : "—", styles: { fontStyle: "bold" } },
        { content: t_totals.deposit_held_amount > 0 ? t_totals.deposit_held_amount.toLocaleString() : "—", styles: { fontStyle: "bold" } },
        { content: t_totals.unpaid_count > 0 ? String(t_totals.unpaid_count) : "—", styles: { fontStyle: "bold" } },
        { content: t_totals.unpaid_amount > 0 ? t_totals.unpaid_amount.toLocaleString() : "—", styles: { fontStyle: "bold" } },
      ],
    ],
    styles: { fontSize: 8, font: fontName },
    headStyles: { fillColor: [40, 40, 40], font: fontName },
    columnStyles: { 7: { halign: "right" }, 8: { halign: "right" } },
  });

  doc.save(`daily-report-${reportYear.value}-${monthLabel}.pdf`);
}
</script>

<template>
  <AppShell>
    <div class="bg-white rounded-lg border border-gray-200 p-6">
      <h1 class="text-lg font-semibold text-gray-900 mb-4">
        {{ t("nav.dailyReport") }}
      </h1>

      <!-- Unreturned deposit summary -->
      <div
        v-if="depositSummary"
        class="mb-5 flex items-center gap-4 rounded-lg border px-4 py-3"
        :class="depositSummary.count > 0 ? 'border-amber-300 bg-amber-50' : 'border-green-200 bg-green-50'"
      >
        <div
          class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full"
          :class="depositSummary.count > 0 ? 'bg-amber-100' : 'bg-green-100'"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" :class="depositSummary.count > 0 ? 'text-amber-600' : 'text-green-600'">
            <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
          </svg>
        </div>
        <div>
          <p class="text-xs font-semibold uppercase tracking-wide" :class="depositSummary.count > 0 ? 'text-amber-700' : 'text-green-700'">
            {{ t("dailyReport.depositsHeld") }}
          </p>
          <p class="mt-0.5 text-sm" :class="depositSummary.count > 0 ? 'text-amber-900' : 'text-green-800'">
            <span class="font-semibold">{{ t("dailyReport.bookingsCount", { n: depositSummary.count }) }}</span>
            &nbsp;·&nbsp;
            <span class="font-semibold">{{ settingsStore.formatPrice(depositSummary.total_amount) }}</span>
            <span class="text-xs ml-2 opacity-70">{{ t("dailyReport.depositsHeldHint") }}</span>
          </p>
        </div>
      </div>

      <div class="flex items-end justify-between gap-4 mb-6">
        <div class="flex items-end gap-4">
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-gray-700">{{ t("adminBookings.year") }}</label>
            <input
              v-model.number="reportYear"
              type="number"
              :min="2000"
              :max="2100"
              class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400 w-28"
            />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-gray-700">{{ t("adminBookings.month") }}</label>
            <select
              v-model.number="reportMonth"
              class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400"
            >
              <option
                v-for="opt in monthOptions"
                :key="opt.value"
                :value="opt.value"
              >
                {{ opt.label }}
              </option>
            </select>
          </div>
        </div>
        <button
          v-if="reportRows && reportRows.length > 0"
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
            <th class="py-2 pr-2 w-6"></th>
            <th class="py-2 pr-4 font-medium">{{ t("dailyReport.date") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminBookings.paymentPaid") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("dailyReport.cashRevenue") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("dailyReport.bankTransferRevenue") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("dailyReport.unspecifiedRevenue") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("dailyReport.dayTotal") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("dailyReport.runningTotal") }}</th>
            <th class="py-2 pr-4 font-medium text-right text-amber-600">{{ t("dailyReport.depositsHeldHeader") }}</th>
            <th class="py-2 pr-4 font-medium text-right text-orange-500">{{ t("dailyReport.unpaidCount") }}</th>
            <th class="py-2 font-medium text-right text-orange-500">{{ t("dailyReport.unpaidAmount") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="!reportRows || reportRows.length === 0">
            <td colspan="11" class="py-8 text-center text-sm text-gray-400">
              {{ t("adminBookings.emptyState") }}
            </td>
          </tr>

          <template v-for="row in reportRows" :key="row.date">
            <!-- Day summary row -->
            <tr
              class="border-b border-gray-100 hover:bg-gray-50 cursor-pointer"
              @click="toggleExpand(row.date)"
            >
              <td class="py-2 pr-2">
                <span v-if="loadingDates.has(row.date)" class="loading loading-spinner loading-xs text-gray-400"></span>
                <svg
                  v-else
                  xmlns="http://www.w3.org/2000/svg"
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  class="text-gray-400 transition-transform"
                  :class="expandedDates.has(row.date) ? 'rotate-90' : ''"
                >
                  <polyline points="9 18 15 12 9 6"/>
                </svg>
              </td>
              <td class="py-2 pr-4 font-mono text-gray-900">{{ settingsStore.formatDate(row.date) }}</td>
              <td class="py-2 pr-4 text-gray-600">{{ row.booking_count }}</td>
              <td class="py-2 pr-4 text-gray-600">{{ settingsStore.formatPrice(row.cash_revenue) }}</td>
              <td class="py-2 pr-4 text-gray-600">{{ settingsStore.formatPrice(row.bank_transfer_revenue) }}</td>
              <td class="py-2 pr-4 text-gray-600">{{ settingsStore.formatPrice(row.unspecified_revenue) }}</td>
              <td class="py-2 pr-4 text-gray-900 font-medium">{{ settingsStore.formatPrice(row.total_revenue) }}</td>
              <td class="py-2 pr-4 text-gray-900">{{ settingsStore.formatPrice(row.cumulative_total) }}</td>
              <td class="py-2 pr-4 text-right">
                <span v-if="row.deposit_held_count > 0" class="inline-flex flex-col items-end leading-tight">
                  <span class="text-xs font-semibold text-amber-600">{{ t("dailyReport.bookingsCount", { n: row.deposit_held_count }) }}</span>
                  <span class="text-xs text-amber-500">{{ settingsStore.formatPrice(row.deposit_held_amount) }}</span>
                </span>
                <span v-else class="text-gray-300 text-xs">—</span>
              </td>
              <td class="py-2 pr-4 text-right">
                <span v-if="row.unpaid_count > 0" class="text-xs font-semibold text-orange-500">{{ row.unpaid_count }}</span>
                <span v-else class="text-gray-300 text-xs">—</span>
              </td>
              <td class="py-2 text-right">
                <span v-if="row.unpaid_amount > 0" class="text-xs text-orange-400">{{ settingsStore.formatPrice(row.unpaid_amount) }}</span>
                <span v-else class="text-gray-300 text-xs">—</span>
              </td>
            </tr>

            <!-- Expanded booking detail -->
            <tr v-if="expandedDates.has(row.date)" :key="row.date + '-detail'">
              <td colspan="11" class="p-0">
                <div class="bg-gray-50 border-b border-gray-200 px-8 py-3">
                  <div class="flex items-center justify-between mb-2">
                    <p class="text-xs font-semibold text-gray-500 uppercase tracking-wide">
                      {{ t("dailyReport.bookingsForDate", { date: row.date }) }}
                    </p>
                    <button
                      v-if="detailCache.get(row.date)?.length"
                      class="flex items-center gap-1.5 rounded bg-gray-800 text-white px-2.5 py-1 text-xs font-medium hover:bg-gray-600 transition-colors"
                      @click.stop="downloadDayPdf(row.date, detailCache.get(row.date)!)"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                        <polyline points="7 10 12 15 17 10"/>
                        <line x1="12" y1="15" x2="12" y2="3"/>
                      </svg>
                      PDF
                    </button>
                  </div>

                  <!-- Loading state -->
                  <div v-if="loadingDates.has(row.date)" class="py-4 text-center text-sm text-gray-400">
                    <span class="loading loading-spinner loading-sm"></span>
                  </div>

                  <!-- Empty state -->
                  <div
                    v-else-if="detailCache.get(row.date)?.length === 0"
                    class="py-2 text-sm text-gray-400"
                  >
                    {{ t("dailyReport.noBookings") }}
                  </div>

                  <!-- Detail table -->
                  <table v-else class="w-full text-xs">
                    <thead>
                      <tr class="text-left text-gray-400 border-b border-gray-200">
                        <th class="pb-1 pr-4 font-medium">{{ t("dailyReport.detailRef") }}</th>
                        <th class="pb-1 pr-4 font-medium">{{ t("dailyReport.detailGuest") }}</th>
                        <th class="pb-1 pr-4 font-medium">{{ t("adminBookings.rooms") }}</th>
                        <th class="pb-1 pr-4 font-medium">{{ t("adminBookings.checkIn") }}</th>
                        <th class="pb-1 pr-4 font-medium">{{ t("adminBookings.checkOut") }}</th>
                        <th class="pb-1 pr-4 font-medium">{{ t("dailyReport.detailPayment") }}</th>
                        <th class="pb-1 pr-4 font-medium">{{ t("adminBookings.status") }}</th>
                        <th class="pb-1 pr-4 font-medium text-right">{{ t("dailyReport.detailAmount") }}</th>
                        <th class="pb-1 font-medium text-right">{{ t("dailyReport.detailDeposit") }}</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr
                        v-for="b in detailCache.get(row.date)"
                        :key="b.booking_ref"
                        class="border-b border-gray-100 last:border-0"
                      >
                        <td class="py-1.5 pr-4 font-mono text-gray-700">{{ b.booking_ref }}</td>
                        <td class="py-1.5 pr-4 text-gray-700">{{ b.customer_name ?? "—" }}</td>
                        <td class="py-1.5 pr-4 text-gray-700">{{ b.rooms || "—" }}</td>
                        <td class="py-1.5 pr-4 text-gray-500">{{ settingsStore.formatDate(b.check_in) }}</td>
                        <td class="py-1.5 pr-4 text-gray-500">{{ settingsStore.formatDate(b.check_out) }}</td>
                        <td class="py-1.5 pr-4">
                          <span class="badge badge-xs" :class="paymentBadgeClass(b.payment_method)">
                            {{ paymentLabel(b.payment_method) }}
                          </span>
                        </td>
                        <td class="py-1.5 pr-4">
                          <span class="badge badge-xs" :class="b.record_type === 'paid' ? 'badge-info' : 'badge-ghost'">
                            {{ b.record_type === 'paid' ? t('adminBookings.paymentPaid') : t('adminBookings.paymentUnpaid') }}
                          </span>
                        </td>
                        <td class="py-1.5 pr-4 text-right font-medium text-gray-900">{{ settingsStore.formatPrice(b.net_revenue) }}</td>
                        <td class="py-1.5 text-right">
                          <span
                            v-if="b.deposit_amount != null && b.deposit_amount > 0"
                            class="inline-flex items-center gap-1 text-xs font-medium"
                            :class="b.deposit_returned ? 'text-green-600' : 'text-amber-600'"
                          >
                            {{ settingsStore.formatPrice(b.deposit_amount) }}
                            <span class="badge badge-xs" :class="b.deposit_returned ? 'badge-success' : 'badge-warning'">
                              {{ b.deposit_returned ? t("adminBookings.depositReturned") : t("dailyReport.depositHeld") }}
                            </span>
                          </span>
                          <span v-else class="text-gray-300">—</span>
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </td>
            </tr>
          </template>

          <!-- Totals row -->
          <tr v-if="reportRows && reportRows.length > 0" class="border-t-2 border-gray-300 font-semibold text-gray-900">
            <td class="py-2 pr-2"></td>
            <td class="py-2 pr-4">{{ t("dailyReport.totals") }}</td>
            <td class="py-2 pr-4">{{ totals.booking_count }}</td>
            <td class="py-2 pr-4">{{ settingsStore.formatPrice(totals.cash_revenue) }}</td>
            <td class="py-2 pr-4">{{ settingsStore.formatPrice(totals.bank_transfer_revenue) }}</td>
            <td class="py-2 pr-4">{{ settingsStore.formatPrice(totals.unspecified_revenue) }}</td>
            <td class="py-2 pr-4">{{ settingsStore.formatPrice(totals.total_revenue) }}</td>
            <td class="py-2 pr-4">—</td>
            <td class="py-2 pr-4 text-right">
              <span v-if="totals.deposit_held_count > 0" class="inline-flex flex-col items-end leading-tight text-amber-600">
                <span>{{ t("dailyReport.bookingsCount", { n: totals.deposit_held_count }) }}</span>
                <span class="text-sm">{{ settingsStore.formatPrice(totals.deposit_held_amount) }}</span>
              </span>
              <span v-else class="text-gray-400 font-normal text-xs">—</span>
            </td>
            <td class="py-2 pr-4 text-right text-orange-500">
              {{ totals.unpaid_count > 0 ? totals.unpaid_count : "—" }}
            </td>
            <td class="py-2 text-right text-orange-500">
              {{ totals.unpaid_amount > 0 ? settingsStore.formatPrice(totals.unpaid_amount) : "—" }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </AppShell>
</template>
