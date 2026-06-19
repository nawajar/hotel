<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { VueDatePicker } from "@vuepic/vue-datepicker";
import { ApiError } from "@/api/client";
import AppShell from "@/components/AppShell.vue";
import type { Booking, BookingDetail } from "@/api/bookings";
import {
  useBookingsQuery,
  useBookingQuery,
  useCreateBookingMutation,
  useUpdateBookingMutation,
  useCancelBookingMutation,
  useCancelBookingRoomMutation,
  useAddExtraServiceMutation,
  useRemoveExtraServiceMutation,
  useUploadDocumentMutation,
  useDeleteDocumentMutation,
  useTodaySummaryQuery,
  useRoomAvailabilityQuery,
} from "@/composables/useBookingsQueries";
import { bookingsApi } from "@/api/bookings";

const { t } = useI18n();

const { data: bookings, isLoading: bookingsLoading } = useBookingsQuery();
const { data: todaySummary } = useTodaySummaryQuery();

// --- Filters & pagination ---
type FilterTab = "today" | "upcoming" | "all";
const activeTab = ref<FilterTab>("today");
const statusFilter = ref<"" | "active" | "cancelled">("");
const paymentFilter = ref<"" | "paid" | "unpaid">("");
const PAGE_SIZE = 15;
const currentPage = ref(1);

const todayDate = new Date().toISOString().slice(0, 10);

const filteredBookings = computed(() => {
  if (!bookings.value) return [];
  let list = bookings.value;

  if (activeTab.value === "today") {
    list = list.filter((b) => {
      const ci = b.check_in.slice(0, 10);
      const co = b.check_out.slice(0, 10);
      return ci <= todayDate && co >= todayDate;
    });
  } else if (activeTab.value === "upcoming") {
    list = list.filter((b) => b.check_in.slice(0, 10) > todayDate);
  }

  if (statusFilter.value) {
    list = list.filter((b) => b.status === statusFilter.value);
  }

  if (paymentFilter.value) {
    list = list.filter((b) => b.payment_status === paymentFilter.value);
  }

  return list;
});

const totalPages = computed(() =>
  Math.max(1, Math.ceil(filteredBookings.value.length / PAGE_SIZE)),
);

const pagedBookings = computed(() => {
  const start = (currentPage.value - 1) * PAGE_SIZE;
  return filteredBookings.value.slice(start, start + PAGE_SIZE);
});

function onTabChange(tab: FilterTab) {
  activeTab.value = tab;
  currentPage.value = 1;
}

function onStatusChange(val: "" | "active" | "cancelled") {
  statusFilter.value = val;
  currentPage.value = 1;
}

function onPaymentFilterChange(val: "" | "paid" | "unpaid") {
  paymentFilter.value = val;
  currentPage.value = 1;
}

// --- Detail dialog ---
const selectedBookingId = ref("");
const detailDialogVisible = ref(false);
const detailError = ref("");
const detailDialogEl = ref<HTMLDialogElement>();
const { data: bookingDetail, isLoading: detailLoading } = useBookingQuery(selectedBookingId);

function openDetail(id: string) {
  selectedBookingId.value = id;
  detailError.value = "";
  detailDialogVisible.value = true;
}

watch(detailDialogVisible, (v) => {
  if (v) nextTick(() => detailDialogEl.value?.showModal());
  else detailDialogEl.value?.close();
});

const cancelBookingMutation = useCancelBookingMutation(selectedBookingId);
const cancelRoomMutation = useCancelBookingRoomMutation();
const addExtraServiceMutation = useAddExtraServiceMutation();
const removeExtraServiceMutation = useRemoveExtraServiceMutation();
const uploadDocumentMutation = useUploadDocumentMutation();
const deleteDocumentMutation = useDeleteDocumentMutation();

const uploadError = ref("");
const isUploading = ref(false);

async function handleFileUpload(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  input.value = "";
  uploadError.value = "";
  isUploading.value = true;
  try {
    await uploadDocumentMutation.mutateAsync({ bookingId: selectedBookingId.value, file });
  } catch (err) {
    uploadError.value = err instanceof ApiError ? err.message : t("adminBookings.genericError");
  } finally {
    isUploading.value = false;
  }
}

async function handleDeleteDocument(docId: string) {
  if (!window.confirm(t("adminBookings.confirmDeleteDocument"))) return;
  uploadError.value = "";
  try {
    await deleteDocumentMutation.mutateAsync({ bookingId: selectedBookingId.value, docId });
  } catch (err) {
    uploadError.value = err instanceof ApiError ? err.message : t("adminBookings.genericError");
  }
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

// --- Form dialog ---
const formDialogVisible = ref(false);
const formDialogEl = ref<HTMLDialogElement>();
const editingBooking = ref<Booking | null>(null);
const errorMessage = ref("");

type DiscountMode = "none" | "amount" | "percentage";

const form = reactive({
  check_in: null as string | null,
  check_out: null as string | null,
  room_ids: [] as string[],
  label: "" as string,
  note: "" as string,
  discount_mode: "none" as DiscountMode,
  discount_value: 0,
  payment_status: false,
  customer_name: "",
  customer_phone: "",
  customer_id_type: "",
  customer_id_number: "",
});

const showCustomer = ref(false);
const extraServiceForm = reactive({ name: "", amount: 0 });
const extraServiceError = ref("");

watch(formDialogVisible, (v) => {
  if (v) nextTick(() => formDialogEl.value?.showModal());
  else formDialogEl.value?.close();
});

const nights = computed(() => {
  if (!form.check_in || !form.check_out) return 0;
  return Math.max(
    0,
    Math.round(
      (new Date(form.check_out).getTime() - new Date(form.check_in).getTime()) / 86400000,
    ),
  );
});

const availabilityCheckIn = computed(() =>
  form.check_in ? `${form.check_in}T14:00:00Z` : "",
);
const availabilityCheckOut = computed(() =>
  form.check_out ? `${form.check_out}T12:00:00Z` : "",
);

const todayForPicker = new Date();
todayForPicker.setHours(0, 0, 0, 0);

const minCheckOut = computed<Date | undefined>(() => {
  if (!form.check_in) return undefined;
  const d = new Date(form.check_in);
  d.setDate(d.getDate() + 1);
  return d;
});


const { data: roomAvailability, isLoading: availabilityLoading } = useRoomAvailabilityQuery(
  availabilityCheckIn,
  availabilityCheckOut,
);

function openAddDialog() {
  editingBooking.value = null;
  form.check_in = null;
  form.check_out = null;
  form.room_ids = [];
  form.label = "";
  form.note = "";
  form.discount_mode = "none";
  form.discount_value = 0;
  form.payment_status = false;
  form.customer_name = "";
  form.customer_phone = "";
  form.customer_id_type = "";
  form.customer_id_number = "";
  showCustomer.value = false;
  errorMessage.value = "";
  formDialogVisible.value = true;
}

function openEditDialog(booking: Booking) {
  editingBooking.value = booking;
  form.check_in = booking.check_in.slice(0, 10);
  form.check_out = booking.check_out.slice(0, 10);
  form.room_ids = [];
  form.label = booking.label ?? "";
  form.note = booking.note ?? "";
  form.discount_mode =
    booking.discount_type === "amount"
      ? "amount"
      : booking.discount_type === "percentage"
        ? "percentage"
        : "none";
  form.discount_value = booking.discount_value ?? 0;
  form.payment_status = booking.payment_status === "paid";
  form.customer_name = booking.customer_name ?? "";
  form.customer_phone = booking.customer_phone ?? "";
  form.customer_id_type = booking.customer_id_type ?? "";
  form.customer_id_number = booking.customer_id_number ?? "";
  showCustomer.value = !!(
    booking.customer_name ||
    booking.customer_phone ||
    booking.customer_id_type ||
    booking.customer_id_number
  );
  errorMessage.value = "";
  formDialogVisible.value = true;
}

function toggleRoom(roomId: string) {
  const idx = form.room_ids.indexOf(roomId);
  if (idx >= 0) form.room_ids.splice(idx, 1);
  else form.room_ids.push(roomId);
}

async function handleSubmit() {
  errorMessage.value = "";
  if (!form.check_in || !form.check_out) {
    errorMessage.value = t("adminBookings.checkIn") + " / " + t("adminBookings.checkOut") + " required";
    return;
  }
  try {
    const check_in = `${form.check_in}T14:00:00Z`;
    const check_out = `${form.check_out}T12:00:00Z`;
    const discount_type = form.discount_mode === "none" ? undefined : form.discount_mode;
    const discount_value = form.discount_mode === "none" ? undefined : form.discount_value;
    const payment_status = form.payment_status ? "paid" : "unpaid";

    if (editingBooking.value) {
      await updateBookingMutation.mutateAsync({
        id: editingBooking.value.id,
        input: {
          check_in, check_out,
          label: form.label || undefined,
          note: form.note || undefined,
          discount_type, discount_value, payment_status,
          customer_name: form.customer_name || undefined,
          customer_phone: form.customer_phone || undefined,
          customer_id_type: form.customer_id_type || undefined,
          customer_id_number: form.customer_id_number || undefined,
        },
      });
    } else {
      await createBookingMutation.mutateAsync({
        check_in, check_out,
        room_ids: form.room_ids,
        label: form.label || undefined,
        note: form.note || undefined,
        discount_type, discount_value, payment_status,
        customer_name: form.customer_name || undefined,
        customer_phone: form.customer_phone || undefined,
        customer_id_type: form.customer_id_type || undefined,
        customer_id_number: form.customer_id_number || undefined,
      });
    }
    formDialogVisible.value = false;
  } catch (err) {
    errorMessage.value = err instanceof ApiError ? err.message : t("adminBookings.genericError");
  }
}

async function handleCancelBooking() {
  if (!window.confirm(t("adminBookings.confirmCancelBooking"))) return;
  detailError.value = "";
  try {
    await cancelBookingMutation.mutateAsync();
    detailDialogVisible.value = false;
  } catch (err) {
    detailError.value = err instanceof ApiError ? err.message : t("adminBookings.genericError");
  }
}

async function handleCancelRoom(roomBookingId: string) {
  if (!window.confirm(t("adminBookings.confirmCancelRoom"))) return;
  detailError.value = "";
  try {
    await cancelRoomMutation.mutateAsync({ bookingId: selectedBookingId.value, roomBookingId });
  } catch (err) {
    detailError.value = err instanceof ApiError ? err.message : t("adminBookings.genericError");
  }
}

async function handleAddExtraService() {
  extraServiceError.value = "";
  try {
    await addExtraServiceMutation.mutateAsync({
      bookingId: selectedBookingId.value,
      input: { name: extraServiceForm.name, amount: extraServiceForm.amount },
    });
    extraServiceForm.name = "";
    extraServiceForm.amount = 0;
  } catch (err) {
    extraServiceError.value = err instanceof ApiError ? err.message : t("adminBookings.genericError");
  }
}

async function handleRemoveExtraService(serviceId: string) {
  if (!window.confirm(t("adminBookings.confirmRemoveService"))) return;
  detailError.value = "";
  try {
    await removeExtraServiceMutation.mutateAsync({ bookingId: selectedBookingId.value, serviceId });
  } catch (err) {
    detailError.value = err instanceof ApiError ? err.message : t("adminBookings.genericError");
  }
}

const createBookingMutation = useCreateBookingMutation();
const updateBookingMutation = useUpdateBookingMutation();

async function handleTogglePayment(detail: BookingDetail) {
  detailError.value = "";
  const b = detail.booking;
  const newStatus = b.payment_status === "paid" ? "unpaid" : "paid";
  try {
    await updateBookingMutation.mutateAsync({
      id: b.id,
      input: {
        check_in: b.check_in, check_out: b.check_out,
        label: b.label ?? undefined,
        note: b.note ?? undefined,
        discount_type: b.discount_type ?? undefined,
        discount_value: b.discount_value ?? undefined,
        payment_status: newStatus,
        customer_name: b.customer_name ?? undefined,
        customer_phone: b.customer_phone ?? undefined,
        customer_id_type: b.customer_id_type ?? undefined,
        customer_id_number: b.customer_id_number ?? undefined,
      },
    });
  } catch (err) {
    detailError.value = err instanceof ApiError ? err.message : t("adminBookings.genericError");
  }
}

function computeTotal(detail: BookingDetail): number {
  const roomTotal = detail.rooms
    .filter((r) => r.status === "active")
    .reduce((s, r) => s + r.price_snapshot, 0);
  const extraTotal = detail.extra_services.reduce((s, x) => s + x.amount, 0);
  const subtotal = roomTotal + extraTotal;
  const b = detail.booking;
  if (!b.discount_type) return subtotal;
  if (b.discount_type === "amount") return Math.max(0, subtotal - (b.discount_value ?? 0));
  return subtotal * (1 - (b.discount_value ?? 0) / 100);
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString();
}

function formatPrice(price: number) {
  return price.toLocaleString();
}

const isPending = computed(
  () => createBookingMutation.isPending.value || updateBookingMutation.isPending.value,
);

// --- Stat panel ---
type StatType = "occupied" | "check_ins" | "check_outs" | "needs_attention";
const statPanel = ref<{ visible: boolean; type: StatType | null }>({ visible: false, type: null });
const statDialogEl = ref<HTMLDialogElement>();

watch(() => statPanel.value.visible, (v) => {
  if (v) nextTick(() => statDialogEl.value?.showModal());
  else statDialogEl.value?.close();
});

const statPanelTitle = computed(() => {
  switch (statPanel.value.type) {
    case "occupied": return t("adminBookings.statOccupied");
    case "check_ins": return t("adminBookings.statCheckInsToday");
    case "check_outs": return t("adminBookings.statCheckOutsToday");
    case "needs_attention": return t("adminBookings.statNeedsAttention");
    default: return "";
  }
});

const statPanelBookings = computed(() => {
  if (!bookings.value || !statPanel.value.type) return [];
  return bookings.value.filter((b) => {
    if (b.status !== "active") return false;
    const ci = b.check_in.slice(0, 10);
    const co = b.check_out.slice(0, 10);
    switch (statPanel.value.type) {
      case "occupied": return ci <= todayDate && co > todayDate;
      case "check_ins": return ci === todayDate;
      case "check_outs": return co === todayDate;
      case "needs_attention": return b.label === "needs_attention";
    }
  });
});

function openStatPanel(type: StatType) {
  statPanel.value = { visible: true, type };
}

function openDetailFromPanel(id: string) {
  statPanel.value.visible = false;
  openDetail(id);
}
</script>

<template>
  <AppShell>
    <!-- Today's snapshot bar -->
    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3 mb-6">
      <!-- Total rooms -->
      <div class="bg-base-100 rounded-xl border border-base-200 shadow-sm px-5 py-4 border-t-2 border-t-base-300">
        <p class="text-[0.65rem] font-semibold uppercase tracking-widest text-base-content/40 h-8 flex items-start">{{ t("adminBookings.statTotalRooms") }}</p>
        <p class="text-3xl font-bold text-base-content">{{ todaySummary?.total_rooms ?? "—" }}</p>
      </div>
      <!-- Available -->
      <div class="bg-base-100 rounded-xl border border-base-200 shadow-sm px-5 py-4 border-t-2 border-t-emerald-400">
        <p class="text-[0.65rem] font-semibold uppercase tracking-widest text-base-content/40 h-8 flex items-start">{{ t("adminBookings.statAvailable") }}</p>
        <p class="text-3xl font-bold text-base-content">{{ todaySummary?.available_now ?? "—" }}</p>
      </div>
      <!-- Occupied -->
      <div
        class="bg-base-100 rounded-xl border border-base-200 shadow-sm px-5 py-4 border-t-2 border-t-amber-400 cursor-pointer hover:bg-base-200/60 transition-colors"
        @click="openStatPanel('occupied')"
      >
        <p class="text-[0.65rem] font-semibold uppercase tracking-widest text-base-content/40 h-8 flex items-start">{{ t("adminBookings.statOccupied") }}</p>
        <p class="text-3xl font-bold text-base-content">{{ todaySummary?.occupied_now ?? "—" }}</p>
      </div>
      <!-- Check-ins today -->
      <div
        class="bg-base-100 rounded-xl border border-base-200 shadow-sm px-5 py-4 border-t-2 border-t-sky-400 cursor-pointer hover:bg-base-200/60 transition-colors"
        @click="openStatPanel('check_ins')"
      >
        <p class="text-[0.65rem] font-semibold uppercase tracking-widest text-base-content/40 h-8 flex items-start">{{ t("adminBookings.statCheckInsToday") }}</p>
        <p class="text-3xl font-bold text-base-content">{{ todaySummary?.check_ins_today ?? "—" }}</p>
      </div>
      <!-- Check-outs today -->
      <div
        class="bg-base-100 rounded-xl border border-base-200 shadow-sm px-5 py-4 border-t-2 border-t-violet-400 cursor-pointer hover:bg-base-200/60 transition-colors"
        @click="openStatPanel('check_outs')"
      >
        <p class="text-[0.65rem] font-semibold uppercase tracking-widest text-base-content/40 h-8 flex items-start">{{ t("adminBookings.statCheckOutsToday") }}</p>
        <p class="text-3xl font-bold text-base-content">{{ todaySummary?.check_outs_today ?? "—" }}</p>
      </div>
      <!-- Needs attention -->
      <div
        class="bg-base-100 rounded-xl border border-base-200 shadow-sm px-5 py-4 border-t-2 cursor-pointer hover:bg-base-200/60 transition-colors"
        :class="todaySummary && todaySummary.needs_attention > 0 ? 'border-t-red-500' : 'border-t-base-300'"
        @click="openStatPanel('needs_attention')"
      >
        <p class="text-[0.65rem] font-semibold uppercase tracking-widest text-base-content/40 h-8 flex items-start">{{ t("adminBookings.statNeedsAttention") }}</p>
        <p class="text-3xl font-bold" :class="todaySummary && todaySummary.needs_attention > 0 ? 'text-red-500' : 'text-base-content'">
          {{ todaySummary?.needs_attention ?? "—" }}
        </p>
      </div>
    </div>

    <!-- Main card -->
    <div class="card bg-base-100 shadow-sm">
      <div class="card-body p-6">
        <!-- Header -->
        <div class="flex items-center justify-between mb-4">
          <h2 class="card-title text-base">{{ t("adminBookings.title") }}</h2>
          <button class="btn btn-sm btn-neutral" @click="openAddDialog">
            {{ t("adminBookings.add") }}
          </button>
        </div>

        <!-- Filter bar -->
        <div class="flex flex-wrap items-center gap-3 mb-4">
          <div class="join">
            <button
              v-for="tab in (['today', 'upcoming', 'all'] as FilterTab[])"
              :key="tab"
              :class="['btn btn-sm join-item', activeTab === tab ? 'btn-neutral' : 'btn-outline']"
              @click="onTabChange(tab)"
            >
              {{ tab === "today" ? t("adminBookings.filterToday") : tab === "upcoming" ? t("adminBookings.filterUpcoming") : t("adminBookings.filterAll") }}
            </button>
          </div>

          <select
            :value="statusFilter"
            class="select select-bordered select-sm"
            @change="onStatusChange(($event.target as HTMLSelectElement).value as '' | 'active' | 'cancelled')"
          >
            <option value="">{{ t("adminBookings.filterStatusAll") }}</option>
            <option value="active">{{ t("adminBookings.filterStatusActive") }}</option>
            <option value="cancelled">{{ t("adminBookings.filterStatusCancelled") }}</option>
          </select>

          <select
            :value="paymentFilter"
            class="select select-bordered select-sm"
            @change="onPaymentFilterChange(($event.target as HTMLSelectElement).value as '' | 'paid' | 'unpaid')"
          >
            <option value="">{{ t("adminBookings.filterPaymentAll") }}</option>
            <option value="paid">{{ t("adminBookings.paymentPaid") }}</option>
            <option value="unpaid">{{ t("adminBookings.paymentUnpaid") }}</option>
          </select>

          <span class="ml-auto text-xs text-base-content/50">
            {{ filteredBookings.length }} {{ filteredBookings.length === 1 ? "booking" : "bookings" }}
          </span>
        </div>

        <!-- Table -->
        <div v-if="bookingsLoading" class="py-8 text-sm text-base-content/40 text-center">…</div>
        <div v-else class="overflow-x-auto">
          <table class="table table-sm">
            <thead>
              <tr>
                <th>{{ t("adminBookings.bookingRef") }}</th>
                <th>{{ t("adminBookings.status") }}</th>
                <th>{{ t("adminBookings.paymentStatus") }}</th>
                <th>{{ t("adminBookings.checkIn") }}</th>
                <th>{{ t("adminBookings.checkOut") }}</th>
                <th>{{ t("adminBookings.customer") }}</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="pagedBookings.length === 0">
                <td colspan="7" class="py-8 text-center text-base-content/40">
                  {{ t("adminBookings.emptyState") }}
                </td>
              </tr>
              <tr v-for="booking in pagedBookings" :key="booking.id" class="hover">
                <td class="font-mono font-medium">{{ booking.booking_ref }}</td>
                <td>
                  <span :class="['badge badge-sm', booking.status === 'active' ? 'badge-success' : 'badge-error']">
                    {{ booking.status === "active" ? t("adminBookings.statusActive") : t("adminBookings.statusCancelled") }}
                  </span>
                </td>
                <td>
                  <span :class="['badge badge-sm', booking.payment_status === 'paid' ? 'badge-info' : 'badge-ghost']">
                    {{ booking.payment_status === "paid" ? t("adminBookings.paymentPaid") : t("adminBookings.paymentUnpaid") }}
                  </span>
                </td>
                <td>{{ formatDate(booking.check_in) }}</td>
                <td>{{ formatDate(booking.check_out) }}</td>
                <td class="text-xs text-base-content/50">{{ booking.customer_name ?? "—" }}</td>
                <td>
                  <div class="flex items-center gap-1">
                    <button class="btn btn-ghost btn-xs" :title="t('adminBookings.view')" @click="openDetail(booking.id)">
                      <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                        <circle cx="12" cy="12" r="3"/>
                      </svg>
                    </button>
                    <button
                      v-if="booking.status === 'active'"
                      class="btn btn-ghost btn-xs"
                      :title="t('adminBookings.edit')"
                      @click="openEditDialog(booking)"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                      </svg>
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Pagination -->
        <div v-if="totalPages > 1" class="flex items-center justify-between mt-4 pt-4 border-t border-base-200">
          <button class="btn btn-sm btn-outline" :disabled="currentPage <= 1" @click="currentPage--">
            {{ t("adminBookings.prevPage") }}
          </button>
          <span class="text-xs text-base-content/50">
            {{ t("adminBookings.pageOf", { page: currentPage, total: totalPages }) }}
          </span>
          <button class="btn btn-sm btn-outline" :disabled="currentPage >= totalPages" @click="currentPage++">
            {{ t("adminBookings.nextPage") }}
          </button>
        </div>
      </div>
    </div>

    <!-- Booking Detail Dialog -->
    <dialog ref="detailDialogEl" class="modal" @close="detailDialogVisible = false">
      <div class="modal-box w-full max-w-2xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between mb-4">
          <span class="font-mono font-semibold">{{ bookingDetail?.booking.booking_ref ?? "…" }}</span>
          <form method="dialog">
            <button class="btn btn-sm btn-circle btn-ghost">✕</button>
          </form>
        </div>

        <div v-if="detailLoading" class="py-8 text-sm text-base-content/40 text-center">…</div>
        <div v-else-if="bookingDetail" class="flex flex-col gap-5">
          <!-- Action buttons -->
          <div class="flex items-center gap-2 flex-wrap">
            <button
              v-if="bookingDetail.booking.status === 'active'"
              class="btn btn-sm btn-outline"
              @click="handleTogglePayment(bookingDetail!)"
            >
              {{ bookingDetail.booking.payment_status === "paid" ? t("adminBookings.markUnpaid") : t("adminBookings.markPaid") }}
            </button>
            <button
              v-if="bookingDetail.booking.status === 'active'"
              class="btn btn-sm btn-outline"
              @click="() => { detailDialogVisible = false; openEditDialog(bookingDetail!.booking); }"
            >
              {{ t("adminBookings.edit") }}
            </button>
            <button
              v-if="bookingDetail.booking.status === 'active'"
              class="btn btn-sm btn-outline btn-error"
              @click="handleCancelBooking"
            >
              {{ t("adminBookings.cancelBooking") }}
            </button>
          </div>

          <div v-if="detailError" class="alert alert-error text-sm">{{ detailError }}</div>

          <!-- Booking info -->
          <div class="grid grid-cols-2 gap-3 text-sm">
            <div>
              <span class="text-base-content/50">{{ t("adminBookings.checkIn") }}:</span>
              <span class="ml-2">{{ formatDate(bookingDetail.booking.check_in) }}</span>
            </div>
            <div>
              <span class="text-base-content/50">{{ t("adminBookings.checkOut") }}:</span>
              <span class="ml-2">{{ formatDate(bookingDetail.booking.check_out) }}</span>
            </div>
            <div>
              <span class="text-base-content/50">{{ t("adminBookings.status") }}:</span>
              <span :class="['badge badge-sm ml-2', bookingDetail.booking.status === 'active' ? 'badge-success' : 'badge-error']">
                {{ bookingDetail.booking.status === "active" ? t("adminBookings.statusActive") : t("adminBookings.statusCancelled") }}
              </span>
            </div>
            <div>
              <span class="text-base-content/50">{{ t("adminBookings.paymentStatus") }}:</span>
              <span :class="['badge badge-sm ml-2', bookingDetail.booking.payment_status === 'paid' ? 'badge-info' : 'badge-ghost']">
                {{ bookingDetail.booking.payment_status === "paid" ? t("adminBookings.paymentPaid") : t("adminBookings.paymentUnpaid") }}
              </span>
            </div>
            <div v-if="bookingDetail.booking.label">
              <span class="text-base-content/50">{{ t("adminBookings.label") }}:</span>
              <span class="ml-2">{{ bookingDetail.booking.label }}</span>
            </div>
            <div v-if="bookingDetail.booking.note">
              <span class="text-base-content/50">{{ t("adminBookings.note") }}:</span>
              <span class="ml-2">{{ bookingDetail.booking.note }}</span>
            </div>
            <div v-if="bookingDetail.booking.discount_type">
              <span class="text-base-content/50">{{ t("adminBookings.discount") }}:</span>
              <span class="ml-2">
                {{ bookingDetail.booking.discount_type === "percentage" ? "%" : "₭" }}{{ bookingDetail.booking.discount_value }}
              </span>
            </div>
            <template v-if="bookingDetail.booking.customer_name || bookingDetail.booking.customer_phone || bookingDetail.booking.customer_id_type">
              <div class="col-span-2">
                <div class="divider my-1 text-xs text-base-content/40">{{ t("adminBookings.customer") }}</div>
              </div>
              <div v-if="bookingDetail.booking.customer_name">
                <span class="text-base-content/50">{{ t("adminBookings.customerName") }}:</span>
                <span class="ml-2">{{ bookingDetail.booking.customer_name }}</span>
              </div>
              <div v-if="bookingDetail.booking.customer_phone">
                <span class="text-base-content/50">{{ t("adminBookings.customerPhone") }}:</span>
                <span class="ml-2">{{ bookingDetail.booking.customer_phone }}</span>
              </div>
              <div v-if="bookingDetail.booking.customer_id_type">
                <span class="text-base-content/50">{{ t("adminBookings.customerIdType") }}:</span>
                <span class="ml-2">{{ bookingDetail.booking.customer_id_type }}</span>
              </div>
              <div v-if="bookingDetail.booking.customer_id_number">
                <span class="text-base-content/50">{{ t("adminBookings.customerIdNumber") }}:</span>
                <span class="ml-2">{{ bookingDetail.booking.customer_id_number }}</span>
              </div>
            </template>
            <div class="col-span-2">
              <div class="flex flex-wrap gap-4 text-xs text-base-content/40 pt-1 border-t border-base-200">
                <span>{{ t("adminBookings.createdAt") }}: <span class="text-base-content/60">{{ new Date(bookingDetail.booking.created_at).toLocaleString() }}</span></span>
                <span>{{ t("adminBookings.createdBy") }}: <span class="text-base-content/60">{{ bookingDetail.booking.created_by_name || "—" }}</span></span>
                <span>{{ t("adminBookings.lastEditedBy") }}: <span class="text-base-content/60">{{ bookingDetail.booking.updated_by_name || "—" }}</span></span>
              </div>
            </div>
          </div>

          <!-- Rooms -->
          <div>
            <div class="divider my-1 text-xs text-base-content/40">{{ t("adminBookings.rooms") }}</div>
            <div class="overflow-x-auto">
              <table class="table table-xs">
                <thead>
                  <tr>
                    <th>{{ t("adminBookings.roomNumber") }}</th>
                    <th>{{ t("adminBookings.roomName") }}</th>
                    <th>{{ t("adminBookings.status") }}</th>
                    <th>{{ t("adminBookings.price") }}</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="room in bookingDetail.rooms" :key="room.id">
                    <td class="font-medium">{{ room.room_number }}</td>
                    <td>{{ room.room_name }}</td>
                    <td>
                      <span :class="['badge badge-xs', room.status === 'active' ? 'badge-success' : 'badge-error']">{{ room.status }}</span>
                    </td>
                    <td>₭{{ formatPrice(room.price_snapshot) }}</td>
                    <td>
                      <button
                        v-if="room.status === 'active' && bookingDetail!.booking.status === 'active'"
                        class="btn btn-xs btn-ghost btn-error"
                        @click="handleCancelRoom(room.id)"
                      >{{ t("adminBookings.cancelRoom") }}</button>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <!-- Extra services -->
          <div>
            <div class="divider my-1 text-xs text-base-content/40">{{ t("adminBookings.extraServices") }}</div>
            <div v-if="bookingDetail.extra_services.length > 0" class="overflow-x-auto mb-3">
              <table class="table table-xs">
                <tbody>
                  <tr v-for="svc in bookingDetail.extra_services" :key="svc.id">
                    <td>{{ svc.name }}</td>
                    <td>₭{{ formatPrice(svc.amount) }}</td>
                    <td>
                      <button class="btn btn-xs btn-ghost btn-error" @click="handleRemoveExtraService(svc.id)">
                        {{ t("adminBookings.removeService") }}
                      </button>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div v-if="bookingDetail.booking.status === 'active'" class="flex items-end gap-2">
              <label class="form-control">
                <div class="label py-0.5"><span class="label-text text-xs">{{ t("adminBookings.serviceNamePlaceholder") }}</span></div>
                <input
                  v-model="extraServiceForm.name"
                  class="input input-bordered input-sm w-40"
                  :placeholder="t('adminBookings.serviceNamePlaceholder')"
                />
              </label>
              <label class="form-control">
                <div class="label py-0.5"><span class="label-text text-xs">{{ t("adminBookings.serviceAmount") }}</span></div>
                <input
                  v-model.number="extraServiceForm.amount"
                  type="number"
                  min="0"
                  class="input input-bordered input-sm w-28"
                />
              </label>
              <button
                class="btn btn-sm btn-neutral"
                :disabled="!extraServiceForm.name || extraServiceForm.amount < 0"
                @click="handleAddExtraService"
              >{{ t("adminBookings.addExtraService") }}</button>
            </div>
            <p v-if="extraServiceError" class="mt-1 text-xs text-error">{{ extraServiceError }}</p>
          </div>

          <!-- Documents -->
          <div>
            <div class="divider my-1 text-xs text-base-content/40">{{ t("adminBookings.documents") }}</div>
            <div v-if="bookingDetail.documents.length > 0" class="flex flex-col gap-1.5 mb-3">
              <div
                v-for="doc in bookingDetail.documents"
                :key="doc.id"
                class="flex items-center gap-3 rounded-lg border border-base-200 bg-base-200/40 px-3 py-2"
              >
                <span class="text-lg leading-none">{{ doc.mime_type === "application/pdf" ? "📄" : "🖼️" }}</span>
                <div class="flex-1 min-w-0">
                  <p class="text-sm truncate">{{ doc.filename }}</p>
                  <p class="text-xs text-base-content/40">{{ formatFileSize(doc.size) }} · {{ t("adminBookings.uploadedBy") }} {{ doc.uploaded_by_name }}</p>
                </div>
                <a
                  :href="bookingsApi.documentDownloadUrl(bookingDetail!.booking.id, doc.id)"
                  target="_blank"
                  rel="noopener"
                  class="btn btn-xs btn-ghost btn-info shrink-0"
                >{{ t("adminBookings.downloading") }}</a>
                <button class="btn btn-xs btn-ghost btn-error shrink-0" @click="handleDeleteDocument(doc.id)">
                  {{ t("adminBookings.deleteDocument") }}
                </button>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <label :class="['btn btn-sm btn-outline cursor-pointer', isUploading ? 'btn-disabled' : '']">
                <span>{{ isUploading ? "…" : t("adminBookings.uploadDocument") }}</span>
                <input type="file" class="sr-only" accept=".jpg,.jpeg,.png,.pdf,.webp,.gif" :disabled="isUploading" @change="handleFileUpload" />
              </label>
              <span class="text-xs text-base-content/40">{{ t("adminBookings.docAllowedTypes") }}</span>
            </div>
            <p v-if="uploadError" class="mt-1 text-xs text-error">{{ uploadError }}</p>
          </div>

          <!-- Total -->
          <div class="flex justify-end pt-3 border-t border-base-200">
            <span class="text-sm font-semibold">
              {{ t("adminBookings.total") }}: ₭{{ formatPrice(computeTotal(bookingDetail)) }}
            </span>
          </div>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button>close</button></form>
    </dialog>

    <!-- Create / Edit Dialog -->
    <dialog ref="formDialogEl" class="modal" @close="formDialogVisible = false">
      <div class="modal-box w-full max-w-xl max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between mb-4">
          <h3 class="font-semibold">
            {{ editingBooking ? t("adminBookings.editTitle") : t("adminBookings.addTitle") }}
          </h3>
          <form method="dialog">
            <button class="btn btn-sm btn-circle btn-ghost">✕</button>
          </form>
        </div>

        <form id="booking-form" class="flex flex-col gap-5" @submit.prevent="handleSubmit">
          <!-- Dates -->
          <div class="grid grid-cols-2 gap-4">
            <label class="form-control">
              <div class="label py-0.5">
                <span class="label-text text-xs font-medium">{{ t("adminBookings.checkIn") }} <span class="text-error ml-0.5">*</span></span>
              </div>
              <VueDatePicker
                v-model="form.check_in"
                model-type="yyyy-MM-dd"
                :time-config="{ enableTimePicker: false }"
                :min-date="editingBooking ? undefined : todayForPicker"
                :max-date="form.check_out ? new Date(form.check_out) : undefined"
                :teleport="formDialogEl ?? true"
                input-class-name="input input-bordered input-sm w-full"
                auto-apply
              />
            </label>
            <label class="form-control">
              <div class="label py-0.5">
                <span class="label-text text-xs font-medium">{{ t("adminBookings.checkOut") }} <span class="text-error ml-0.5">*</span></span>
              </div>
              <VueDatePicker
                v-model="form.check_out"
                model-type="yyyy-MM-dd"
                :time-config="{ enableTimePicker: false }"
                :min-date="minCheckOut ?? (editingBooking ? undefined : todayForPicker)"
                :teleport="formDialogEl ?? true"
                input-class-name="input input-bordered input-sm w-full"
                auto-apply
              />
            </label>
          </div>
          <p v-if="nights > 0" class="text-xs text-base-content/50 -mt-3">
            {{ nights }} {{ t("adminBookings.nights") }}
          </p>

          <!-- Room selection (only for new bookings) -->
          <div v-if="!editingBooking" class="flex flex-col gap-1.5">
            <div class="flex items-center justify-between">
              <span class="label-text text-xs font-medium">
                {{ t("adminBookings.rooms") }} <span class="text-error ml-0.5">*</span>
              </span>
              <span v-if="roomAvailability" class="text-xs text-base-content/50">
                {{ roomAvailability.filter((r) => r.is_available).length }} {{ t("adminBookings.roomsAvailableOf") }} {{ roomAvailability.length }}
              </span>
            </div>
            <div v-if="!availabilityCheckIn || !availabilityCheckOut" class="rounded-lg border-2 border-dashed border-base-300 p-6 text-center text-sm text-base-content/40">
              {{ t("adminBookings.pickDatesFirst") }}
            </div>
            <div v-else-if="availabilityLoading" class="rounded-lg border border-base-300 p-6 text-center text-sm text-base-content/40">
              {{ t("adminBookings.checkingAvailability") }}
            </div>
            <div v-else class="grid grid-cols-2 gap-2 max-h-64 overflow-y-auto pr-1">
              <button
                v-for="room in roomAvailability"
                :key="room.id"
                type="button"
                :disabled="!room.is_available"
                :class="[
                  'text-left rounded-lg border-2 p-3 transition-all',
                  form.room_ids.includes(room.id)
                    ? 'border-neutral bg-neutral text-neutral-content'
                    : room.is_available
                      ? 'border-base-300 bg-base-100 hover:border-base-content/40 cursor-pointer'
                      : 'border-base-200 bg-base-200 opacity-50 cursor-not-allowed',
                ]"
                @click="room.is_available && toggleRoom(room.id)"
              >
                <div class="flex items-start justify-between gap-1">
                  <span class="text-sm font-semibold leading-tight">{{ t("adminBookings.roomNo") }} {{ room.room_number }}</span>
                  <span v-if="form.room_ids.includes(room.id)" class="text-xs bg-white/20 rounded px-1">✓</span>
                  <span v-else-if="!room.is_available" class="text-xs text-error shrink-0">{{ t("adminBookings.booked") }}</span>
                </div>
                <div class="text-xs mt-0.5 truncate" :class="form.room_ids.includes(room.id) ? 'opacity-70' : 'text-base-content/50'">
                  {{ room.room_name }}
                </div>
                <div class="text-xs mt-1 font-medium" :class="form.room_ids.includes(room.id) ? 'opacity-90' : ''">
                  ₭{{ formatPrice(room.price) }}
                </div>
              </button>
            </div>
          </div>

          <!-- Label -->
          <label class="form-control">
            <div class="label py-0.5"><span class="label-text text-xs font-medium">{{ t("adminBookings.label") }}</span></div>
            <select v-model="form.label" class="select select-bordered select-sm w-full">
              <option value="">{{ t("adminBookings.labelNone") }}</option>
              <option value="check_in">{{ t("adminBookings.labelCheckIn") }}</option>
              <option value="check_out">{{ t("adminBookings.labelCheckOut") }}</option>
              <option value="needs_attention">{{ t("adminBookings.labelNeedsAttention") }}</option>
            </select>
          </label>

          <!-- Note -->
          <label class="form-control">
            <div class="label py-0.5">
              <span class="label-text text-xs font-medium">{{ t("adminBookings.note") }}</span>
              <span class="label-text-alt text-xs text-base-content/40">({{ t("adminBookings.optional") }})</span>
            </div>
            <textarea
              v-model="form.note"
              class="textarea textarea-bordered textarea-sm w-full resize-none"
              rows="2"
              :placeholder="t('adminBookings.notePlaceholder')"
            />
          </label>

          <!-- Discount -->
          <div class="flex flex-col gap-2">
            <span class="label-text text-xs font-medium">{{ t("adminBookings.discount") }}</span>
            <div class="flex items-center gap-4">
              <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                <input v-model="form.discount_mode" type="radio" class="radio radio-sm" value="none" />
                {{ t("adminBookings.discountNone") }}
              </label>
              <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                <input v-model="form.discount_mode" type="radio" class="radio radio-sm" value="amount" />
                {{ t("adminBookings.discountAmount") }}
              </label>
              <label class="flex items-center gap-1.5 text-sm cursor-pointer">
                <input v-model="form.discount_mode" type="radio" class="radio radio-sm" value="percentage" />
                {{ t("adminBookings.discountPercentage") }}
              </label>
            </div>
            <div v-if="form.discount_mode !== 'none'" class="flex items-center gap-2">
              <span class="text-sm text-base-content/50">{{ form.discount_mode === "percentage" ? "%" : "₭" }}</span>
              <input
                v-model.number="form.discount_value"
                type="number"
                min="0"
                :max="form.discount_mode === 'percentage' ? 100 : undefined"
                class="input input-bordered input-sm w-36"
              />
            </div>
          </div>

          <!-- Payment status -->
          <label class="form-control">
            <div class="label py-0.5"><span class="label-text text-xs font-medium">{{ t("adminBookings.paymentStatus") }}</span></div>
            <div class="flex items-center gap-3">
              <input v-model="form.payment_status" type="checkbox" class="toggle toggle-sm" />
              <span class="text-sm" :class="form.payment_status ? '' : 'text-base-content/40'">
                {{ form.payment_status ? t("adminBookings.paymentPaid") : t("adminBookings.paymentUnpaid") }}
              </span>
            </div>
          </label>

          <!-- Customer info toggle -->
          <div>
            <button type="button" class="btn btn-xs btn-ghost gap-1" @click="showCustomer = !showCustomer">
              {{ showCustomer ? "▾" : "▸" }} {{ t("adminBookings.addCustomer") }}
            </button>
            <div v-if="showCustomer" class="mt-3 grid grid-cols-2 gap-4">
              <label class="form-control">
                <div class="label py-0.5"><span class="label-text text-xs font-medium">{{ t("adminBookings.customerName") }}</span></div>
                <input v-model="form.customer_name" class="input input-bordered input-sm w-full" />
              </label>
              <label class="form-control">
                <div class="label py-0.5"><span class="label-text text-xs font-medium">{{ t("adminBookings.customerPhone") }}</span></div>
                <input v-model="form.customer_phone" class="input input-bordered input-sm w-full" />
              </label>
              <label class="form-control">
                <div class="label py-0.5"><span class="label-text text-xs font-medium">{{ t("adminBookings.customerIdType") }}</span></div>
                <input v-model="form.customer_id_type" class="input input-bordered input-sm w-full" />
              </label>
              <label class="form-control">
                <div class="label py-0.5"><span class="label-text text-xs font-medium">{{ t("adminBookings.customerIdNumber") }}</span></div>
                <input v-model="form.customer_id_number" class="input input-bordered input-sm w-full" />
              </label>
            </div>
          </div>

          <div v-if="errorMessage" class="alert alert-error text-sm">{{ errorMessage }}</div>
        </form>

        <!-- Footer -->
        <div class="modal-action mt-4 pt-4 border-t border-base-200">
          <form method="dialog">
            <button class="btn btn-sm btn-outline">{{ t("adminBookings.cancel") }}</button>
          </form>
          <button
            type="submit"
            form="booking-form"
            class="btn btn-sm btn-neutral"
            :class="isPending ? 'loading' : ''"
            :disabled="isPending"
          >
            {{ t("adminBookings.save") }}
          </button>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button>close</button></form>
    </dialog>

    <!-- Stat panel -->
    <dialog ref="statDialogEl" class="modal" @close="statPanel.visible = false">
      <div class="modal-box w-full max-w-md max-h-[80vh] flex flex-col">
        <div class="flex items-center justify-between mb-4">
          <h3 class="font-semibold">{{ statPanelTitle }}</h3>
          <form method="dialog">
            <button class="btn btn-sm btn-circle btn-ghost">✕</button>
          </form>
        </div>
        <div class="overflow-y-auto flex-1">
          <div v-if="statPanelBookings.length === 0" class="py-8 text-center text-sm text-base-content/40">
            {{ t("adminBookings.noBookings") }}
          </div>
          <div v-else class="flex flex-col gap-2">
            <button
              v-for="b in statPanelBookings"
              :key="b.id"
              class="w-full text-left card card-compact card-bordered hover:bg-base-200 transition-colors"
              @click="openDetailFromPanel(b.id)"
            >
              <div class="card-body">
                <div class="flex items-center justify-between gap-2">
                  <span class="font-mono text-xs font-semibold badge badge-ghost">{{ b.booking_ref }}</span>
                  <span :class="['badge badge-sm', b.payment_status === 'paid' ? 'badge-success' : 'badge-warning']">
                    {{ b.payment_status === 'paid' ? t('adminBookings.paid') : t('adminBookings.unpaid') }}
                  </span>
                </div>
                <p class="text-sm font-medium">{{ b.customer_name || "—" }}</p>
                <p class="text-xs text-base-content/40">{{ formatDate(b.check_in) }} → {{ formatDate(b.check_out) }}</p>
              </div>
            </button>
          </div>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button>close</button></form>
    </dialog>
  </AppShell>
</template>

