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
      <div class="bg-white rounded-lg border border-gray-200 p-4">
        <p class="text-xs text-gray-500 mb-1">{{ t("adminBookings.statTotalRooms") }}</p>
        <p class="text-2xl font-bold text-gray-900">{{ todaySummary?.total_rooms ?? "—" }}</p>
      </div>
      <div class="bg-green-50 rounded-lg border border-green-200 p-4">
        <p class="text-xs text-green-700 mb-1">{{ t("adminBookings.statAvailable") }}</p>
        <p class="text-2xl font-bold text-green-700">{{ todaySummary?.available_now ?? "—" }}</p>
      </div>
      <button class="bg-orange-50 rounded-lg border border-orange-200 p-4 text-left hover:bg-orange-100 transition-colors" @click="openStatPanel('occupied')">
        <p class="text-xs text-orange-700 mb-1">{{ t("adminBookings.statOccupied") }}</p>
        <p class="text-2xl font-bold text-orange-700">{{ todaySummary?.occupied_now ?? "—" }}</p>
      </button>
      <button class="bg-blue-50 rounded-lg border border-blue-200 p-4 text-left hover:bg-blue-100 transition-colors" @click="openStatPanel('check_ins')">
        <p class="text-xs text-blue-700 mb-1">{{ t("adminBookings.statCheckInsToday") }}</p>
        <p class="text-2xl font-bold text-blue-700">{{ todaySummary?.check_ins_today ?? "—" }}</p>
      </button>
      <button class="bg-purple-50 rounded-lg border border-purple-200 p-4 text-left hover:bg-purple-100 transition-colors" @click="openStatPanel('check_outs')">
        <p class="text-xs text-purple-700 mb-1">{{ t("adminBookings.statCheckOutsToday") }}</p>
        <p class="text-2xl font-bold text-purple-700">{{ todaySummary?.check_outs_today ?? "—" }}</p>
      </button>
      <button
        :class="[
          'rounded-lg border p-4 text-left transition-colors',
          todaySummary && todaySummary.needs_attention > 0
            ? 'bg-red-50 border-red-300 hover:bg-red-100'
            : 'bg-gray-50 border-gray-200 hover:bg-gray-100',
        ]"
        @click="openStatPanel('needs_attention')"
      >
        <p :class="['text-xs mb-1', todaySummary && todaySummary.needs_attention > 0 ? 'text-red-700' : 'text-gray-500']">
          {{ t("adminBookings.statNeedsAttention") }}
        </p>
        <p :class="['text-2xl font-bold', todaySummary && todaySummary.needs_attention > 0 ? 'text-red-700' : 'text-gray-400']">
          {{ todaySummary?.needs_attention ?? "—" }}
        </p>
      </button>
    </div>

    <div class="bg-white rounded-lg border border-gray-200 p-6">
      <!-- Header -->
      <div class="flex items-center justify-between mb-4">
        <h1 class="text-lg font-semibold text-gray-900">{{ t("adminBookings.title") }}</h1>
        <button class="btn btn-sm bg-gray-900 text-white hover:bg-gray-700 border-none" @click="openAddDialog">
          {{ t("adminBookings.add") }}
        </button>
      </div>

      <!-- Filter bar -->
      <div class="flex flex-wrap items-center gap-3 mb-4">
        <div class="flex rounded-lg border border-gray-200 overflow-hidden">
          <button
            v-for="tab in (['today', 'upcoming', 'all'] as FilterTab[])"
            :key="tab"
            :class="[
              'px-4 py-1.5 text-sm font-medium transition-colors',
              activeTab === tab ? 'bg-gray-900 text-white' : 'bg-white text-gray-600 hover:bg-gray-50',
            ]"
            @click="onTabChange(tab)"
          >
            {{ tab === "today" ? t("adminBookings.filterToday") : tab === "upcoming" ? t("adminBookings.filterUpcoming") : t("adminBookings.filterAll") }}
          </button>
        </div>

        <select
          :value="statusFilter"
          class="select select-bordered select-sm text-sm"
          @change="onStatusChange(($event.target as HTMLSelectElement).value as '' | 'active' | 'cancelled')"
        >
          <option value="">{{ t("adminBookings.filterStatusAll") }}</option>
          <option value="active">{{ t("adminBookings.filterStatusActive") }}</option>
          <option value="cancelled">{{ t("adminBookings.filterStatusCancelled") }}</option>
        </select>

        <span class="ml-auto text-xs text-gray-400">
          {{ filteredBookings.length }} {{ filteredBookings.length === 1 ? "booking" : "bookings" }}
        </span>
      </div>

      <!-- Table -->
      <div v-if="bookingsLoading" class="py-8 text-sm text-gray-400 text-center">…</div>
      <table v-else class="w-full text-sm">
        <thead>
          <tr class="text-left text-gray-500 border-b border-gray-200">
            <th class="py-2 pr-4 font-medium">{{ t("adminBookings.bookingRef") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminBookings.status") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminBookings.paymentStatus") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminBookings.checkIn") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminBookings.checkOut") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminBookings.customer") }}</th>
            <th class="py-2 font-medium"></th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="pagedBookings.length === 0">
            <td colspan="7" class="py-8 text-center text-sm text-gray-400">
              {{ t("adminBookings.emptyState") }}
            </td>
          </tr>
          <tr
            v-for="booking in pagedBookings"
            :key="booking.id"
            class="border-b border-gray-100 hover:bg-gray-50 transition-colors"
          >
            <td class="py-2 pr-4 font-mono text-gray-900">{{ booking.booking_ref }}</td>
            <td class="py-2 pr-4">
              <span :class="['badge badge-sm', booking.status === 'active' ? 'badge-success' : 'badge-error']">
                {{ booking.status === "active" ? t("adminBookings.statusActive") : t("adminBookings.statusCancelled") }}
              </span>
            </td>
            <td class="py-2 pr-4">
              <span :class="['badge badge-sm', booking.payment_status === 'paid' ? 'badge-info' : 'badge-ghost']">
                {{ booking.payment_status === "paid" ? t("adminBookings.paymentPaid") : t("adminBookings.paymentUnpaid") }}
              </span>
            </td>
            <td class="py-2 pr-4 text-gray-600">{{ formatDate(booking.check_in) }}</td>
            <td class="py-2 pr-4 text-gray-600">{{ formatDate(booking.check_out) }}</td>
            <td class="py-2 pr-4 text-gray-500 text-xs">{{ booking.customer_name ?? "—" }}</td>
            <td class="py-2 flex items-center gap-1">
              <button class="btn btn-ghost btn-xs text-gray-400 hover:text-blue-600" :title="t('adminBookings.view')" @click="openDetail(booking.id)">
                <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                  <circle cx="12" cy="12" r="3"/>
                </svg>
              </button>
              <button
                v-if="booking.status === 'active'"
                class="btn btn-ghost btn-xs text-gray-400 hover:text-gray-700"
                :title="t('adminBookings.edit')"
                @click="openEditDialog(booking)"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                </svg>
              </button>
            </td>
          </tr>
        </tbody>
      </table>

      <!-- Pagination -->
      <div v-if="totalPages > 1" class="flex items-center justify-between mt-4 pt-4 border-t border-gray-100">
        <button class="btn btn-sm btn-outline" :disabled="currentPage <= 1" @click="currentPage--">
          {{ t("adminBookings.prevPage") }}
        </button>
        <span class="text-xs text-gray-500">
          {{ t("adminBookings.pageOf", { page: currentPage, total: totalPages }) }}
        </span>
        <button class="btn btn-sm btn-outline" :disabled="currentPage >= totalPages" @click="currentPage++">
          {{ t("adminBookings.nextPage") }}
        </button>
      </div>
    </div>

    <!-- Booking Detail Dialog -->
    <dialog ref="detailDialogEl" class="modal" @close="detailDialogVisible = false">
      <div class="modal-box w-full max-w-2xl max-h-[90vh] overflow-y-auto">
        <!-- Header -->
        <div class="flex items-center justify-between mb-4">
          <span class="font-mono font-semibold text-gray-900">
            {{ bookingDetail?.booking.booking_ref ?? "…" }}
          </span>
          <form method="dialog">
            <button class="btn btn-sm btn-circle btn-ghost">✕</button>
          </form>
        </div>

        <div v-if="detailLoading" class="py-8 text-sm text-gray-400 text-center">…</div>
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
              <span class="text-gray-500">{{ t("adminBookings.checkIn") }}:</span>
              <span class="ml-2 text-gray-900">{{ formatDate(bookingDetail.booking.check_in) }}</span>
            </div>
            <div>
              <span class="text-gray-500">{{ t("adminBookings.checkOut") }}:</span>
              <span class="ml-2 text-gray-900">{{ formatDate(bookingDetail.booking.check_out) }}</span>
            </div>
            <div>
              <span class="text-gray-500">{{ t("adminBookings.status") }}:</span>
              <span :class="['badge badge-sm ml-2', bookingDetail.booking.status === 'active' ? 'badge-success' : 'badge-error']">
                {{ bookingDetail.booking.status === "active" ? t("adminBookings.statusActive") : t("adminBookings.statusCancelled") }}
              </span>
            </div>
            <div>
              <span class="text-gray-500">{{ t("adminBookings.paymentStatus") }}:</span>
              <span :class="['badge badge-sm ml-2', bookingDetail.booking.payment_status === 'paid' ? 'badge-info' : 'badge-ghost']">
                {{ bookingDetail.booking.payment_status === "paid" ? t("adminBookings.paymentPaid") : t("adminBookings.paymentUnpaid") }}
              </span>
            </div>
            <div v-if="bookingDetail.booking.label">
              <span class="text-gray-500">{{ t("adminBookings.label") }}:</span>
              <span class="ml-2 text-gray-900">{{ bookingDetail.booking.label }}</span>
            </div>
            <div v-if="bookingDetail.booking.note">
              <span class="text-gray-500">{{ t("adminBookings.note") }}:</span>
              <span class="ml-2 text-gray-900">{{ bookingDetail.booking.note }}</span>
            </div>
            <div v-if="bookingDetail.booking.discount_type">
              <span class="text-gray-500">{{ t("adminBookings.discount") }}:</span>
              <span class="ml-2 text-gray-900">
                {{ bookingDetail.booking.discount_type === "percentage" ? "%" : "₭" }}{{ bookingDetail.booking.discount_value }}
              </span>
            </div>
            <template v-if="bookingDetail.booking.customer_name || bookingDetail.booking.customer_phone || bookingDetail.booking.customer_id_type">
              <div class="col-span-2 pt-1">
                <span class="text-xs font-semibold text-gray-500 uppercase tracking-wider">{{ t("adminBookings.customer") }}</span>
              </div>
              <div v-if="bookingDetail.booking.customer_name">
                <span class="text-gray-500">{{ t("adminBookings.customerName") }}:</span>
                <span class="ml-2 text-gray-900">{{ bookingDetail.booking.customer_name }}</span>
              </div>
              <div v-if="bookingDetail.booking.customer_phone">
                <span class="text-gray-500">{{ t("adminBookings.customerPhone") }}:</span>
                <span class="ml-2 text-gray-900">{{ bookingDetail.booking.customer_phone }}</span>
              </div>
              <div v-if="bookingDetail.booking.customer_id_type">
                <span class="text-gray-500">{{ t("adminBookings.customerIdType") }}:</span>
                <span class="ml-2 text-gray-900">{{ bookingDetail.booking.customer_id_type }}</span>
              </div>
              <div v-if="bookingDetail.booking.customer_id_number">
                <span class="text-gray-500">{{ t("adminBookings.customerIdNumber") }}:</span>
                <span class="ml-2 text-gray-900">{{ bookingDetail.booking.customer_id_number }}</span>
              </div>
            </template>
            <div class="col-span-2 pt-2 border-t border-gray-100">
              <div class="flex flex-wrap gap-4 text-xs text-gray-400">
                <span>{{ t("adminBookings.createdBy") }}: <span class="text-gray-600">{{ bookingDetail.booking.created_by_name || "—" }}</span></span>
                <span>{{ t("adminBookings.lastEditedBy") }}: <span class="text-gray-600">{{ bookingDetail.booking.updated_by_name || "—" }}</span></span>
              </div>
            </div>
          </div>

          <!-- Rooms -->
          <div>
            <h3 class="text-sm font-medium text-gray-700 mb-2">{{ t("adminBookings.rooms") }}</h3>
            <table class="w-full text-sm">
              <thead>
                <tr class="text-left text-gray-500 border-b border-gray-100">
                  <th class="py-1.5 pr-4 font-medium">{{ t("adminBookings.roomNumber") }}</th>
                  <th class="py-1.5 pr-4 font-medium">{{ t("adminBookings.roomName") }}</th>
                  <th class="py-1.5 pr-4 font-medium">{{ t("adminBookings.status") }}</th>
                  <th class="py-1.5 pr-4 font-medium">{{ t("adminBookings.price") }}</th>
                  <th class="py-1.5 font-medium"></th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="room in bookingDetail.rooms" :key="room.id" class="border-b border-gray-50">
                  <td class="py-1.5 pr-4 text-gray-900">{{ room.room_number }}</td>
                  <td class="py-1.5 pr-4 text-gray-600">{{ room.room_name }}</td>
                  <td class="py-1.5 pr-4">
                    <span :class="['badge badge-sm', room.status === 'active' ? 'badge-success' : 'badge-error']">{{ room.status }}</span>
                  </td>
                  <td class="py-1.5 pr-4 text-gray-600">₭{{ formatPrice(room.price_snapshot) }}</td>
                  <td class="py-1.5">
                    <button
                      v-if="room.status === 'active' && bookingDetail!.booking.status === 'active'"
                      class="text-xs text-red-600 hover:text-red-800"
                      @click="handleCancelRoom(room.id)"
                    >{{ t("adminBookings.cancelRoom") }}</button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Extra services -->
          <div>
            <h3 class="text-sm font-medium text-gray-700 mb-2">{{ t("adminBookings.extraServices") }}</h3>
            <table v-if="bookingDetail.extra_services.length > 0" class="w-full text-sm mb-3">
              <tbody>
                <tr v-for="svc in bookingDetail.extra_services" :key="svc.id" class="border-b border-gray-50">
                  <td class="py-1.5 pr-4 text-gray-900">{{ svc.name }}</td>
                  <td class="py-1.5 pr-4 text-gray-600">₭{{ formatPrice(svc.amount) }}</td>
                  <td class="py-1.5">
                    <button class="text-xs text-red-600 hover:text-red-800" @click="handleRemoveExtraService(svc.id)">
                      {{ t("adminBookings.removeService") }}
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
            <div v-if="bookingDetail.booking.status === 'active'" class="flex items-end gap-2 mt-2">
              <div class="flex flex-col gap-1">
                <label class="text-xs text-gray-500">{{ t("adminBookings.serviceNamePlaceholder") }}</label>
                <input
                  v-model="extraServiceForm.name"
                  class="input input-bordered input-sm w-40"
                  :placeholder="t('adminBookings.serviceNamePlaceholder')"
                />
              </div>
              <div class="flex flex-col gap-1">
                <label class="text-xs text-gray-500">{{ t("adminBookings.serviceAmount") }}</label>
                <input
                  v-model.number="extraServiceForm.amount"
                  type="number"
                  min="0"
                  class="input input-bordered input-sm w-28"
                />
              </div>
              <button
                class="btn btn-sm bg-gray-900 text-white hover:bg-gray-700 border-none"
                :disabled="!extraServiceForm.name || extraServiceForm.amount < 0"
                @click="handleAddExtraService"
              >{{ t("adminBookings.addExtraService") }}</button>
            </div>
            <p v-if="extraServiceError" class="mt-1 text-xs text-red-600">{{ extraServiceError }}</p>
          </div>

          <!-- Documents -->
          <div>
            <h3 class="text-sm font-medium text-gray-700 mb-2">{{ t("adminBookings.documents") }}</h3>
            <div v-if="bookingDetail.documents.length > 0" class="flex flex-col gap-1.5 mb-3">
              <div
                v-for="doc in bookingDetail.documents"
                :key="doc.id"
                class="flex items-center gap-3 rounded-lg border border-gray-100 bg-gray-50 px-3 py-2"
              >
                <span class="text-lg leading-none">{{ doc.mime_type === "application/pdf" ? "📄" : "🖼️" }}</span>
                <div class="flex-1 min-w-0">
                  <p class="text-sm text-gray-900 truncate">{{ doc.filename }}</p>
                  <p class="text-xs text-gray-400">{{ formatFileSize(doc.size) }} · {{ t("adminBookings.uploadedBy") }} {{ doc.uploaded_by_name }}</p>
                </div>
                <a :href="bookingsApi.documentDownloadUrl(bookingDetail!.booking.id, doc.id)" target="_blank" rel="noopener" class="text-xs text-blue-600 hover:text-blue-800 shrink-0">
                  {{ t("adminBookings.downloading") }}
                </a>
                <button class="text-xs text-red-500 hover:text-red-700 shrink-0" @click="handleDeleteDocument(doc.id)">
                  {{ t("adminBookings.deleteDocument") }}
                </button>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <label :class="['btn btn-sm btn-outline cursor-pointer', isUploading ? 'btn-disabled' : '']">
                <span>{{ isUploading ? "…" : t("adminBookings.uploadDocument") }}</span>
                <input type="file" class="sr-only" accept=".jpg,.jpeg,.png,.pdf,.webp,.gif" :disabled="isUploading" @change="handleFileUpload" />
              </label>
              <span class="text-xs text-gray-400">{{ t("adminBookings.docAllowedTypes") }}</span>
            </div>
            <p v-if="uploadError" class="mt-1 text-xs text-red-600">{{ uploadError }}</p>
          </div>

          <!-- Total -->
          <div class="flex justify-end border-t border-gray-100 pt-4">
            <span class="text-sm font-semibold text-gray-900">
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
        <!-- Header -->
        <div class="flex items-center justify-between mb-4">
          <h3 class="font-semibold text-gray-900">
            {{ editingBooking ? t("adminBookings.editTitle") : t("adminBookings.addTitle") }}
          </h3>
          <form method="dialog">
            <button class="btn btn-sm btn-circle btn-ghost">✕</button>
          </form>
        </div>

        <form id="booking-form" class="flex flex-col gap-5" @submit.prevent="handleSubmit">
          <!-- Dates -->
          <div class="grid grid-cols-2 gap-4">
            <div class="flex flex-col gap-1.5">
              <label class="text-xs font-medium text-gray-700">
                {{ t("adminBookings.checkIn") }} <span class="text-red-500 ml-0.5">*</span>
              </label>
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
            </div>
            <div class="flex flex-col gap-1.5">
              <label class="text-xs font-medium text-gray-700">
                {{ t("adminBookings.checkOut") }} <span class="text-red-500 ml-0.5">*</span>
              </label>
              <VueDatePicker
                v-model="form.check_out"
                model-type="yyyy-MM-dd"
                :time-config="{ enableTimePicker: false }"
                :min-date="minCheckOut ?? (editingBooking ? undefined : todayForPicker)"
                :teleport="formDialogEl ?? true"
                input-class-name="input input-bordered input-sm w-full"
                auto-apply
              />
            </div>
          </div>
          <p v-if="nights > 0" class="text-xs text-gray-500 -mt-3">
            {{ nights }} {{ t("adminBookings.nights") }}
          </p>

          <!-- Room selection (only for new bookings) -->
          <div v-if="!editingBooking" class="flex flex-col gap-1.5">
            <div class="flex items-center justify-between">
              <label class="text-xs font-medium text-gray-700">
                {{ t("adminBookings.rooms") }} <span class="text-red-500 ml-0.5">*</span>
              </label>
              <span v-if="roomAvailability" class="text-xs text-gray-500">
                {{ roomAvailability.filter((r) => r.is_available).length }} {{ t("adminBookings.roomsAvailableOf") }} {{ roomAvailability.length }}
              </span>
            </div>
            <div v-if="!availabilityCheckIn || !availabilityCheckOut" class="rounded-lg border-2 border-dashed border-gray-200 p-6 text-center text-sm text-gray-400">
              {{ t("adminBookings.pickDatesFirst") }}
            </div>
            <div v-else-if="availabilityLoading" class="rounded-lg border border-gray-200 p-6 text-center text-sm text-gray-400">
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
                    ? 'border-gray-900 bg-gray-900 text-white'
                    : room.is_available
                      ? 'border-gray-200 bg-white hover:border-gray-400 cursor-pointer'
                      : 'border-gray-100 bg-gray-50 opacity-50 cursor-not-allowed',
                ]"
                @click="room.is_available && toggleRoom(room.id)"
              >
                <div class="flex items-start justify-between gap-1">
                  <span class="text-sm font-semibold leading-tight">{{ t("adminBookings.roomNo") }} {{ room.room_number }}</span>
                  <span v-if="form.room_ids.includes(room.id)" class="text-xs bg-white/20 rounded px-1">✓</span>
                  <span v-else-if="!room.is_available" class="text-xs text-red-500 shrink-0">{{ t("adminBookings.booked") }}</span>
                </div>
                <div class="text-xs mt-0.5 truncate" :class="form.room_ids.includes(room.id) ? 'text-white/70' : 'text-gray-500'">
                  {{ room.room_name }}
                </div>
                <div class="text-xs mt-1 font-medium" :class="form.room_ids.includes(room.id) ? 'text-white/90' : 'text-gray-700'">
                  ₭{{ formatPrice(room.price) }}
                </div>
              </button>
            </div>
          </div>

          <!-- Label -->
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-gray-700">{{ t("adminBookings.label") }}</label>
            <select v-model="form.label" class="select select-bordered select-sm w-full text-sm">
              <option value="">{{ t("adminBookings.labelNone") }}</option>
              <option value="check_in">{{ t("adminBookings.labelCheckIn") }}</option>
              <option value="check_out">{{ t("adminBookings.labelCheckOut") }}</option>
              <option value="needs_attention">{{ t("adminBookings.labelNeedsAttention") }}</option>
            </select>
          </div>

          <!-- Note -->
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-gray-700">
              {{ t("adminBookings.note") }}
              <span class="ml-1 text-xs font-normal text-gray-400">({{ t("adminBookings.optional") }})</span>
            </label>
            <textarea
              v-model="form.note"
              class="textarea textarea-bordered textarea-sm w-full text-sm resize-none"
              rows="2"
              :placeholder="t('adminBookings.notePlaceholder')"
            />
          </div>

          <!-- Discount -->
          <div class="flex flex-col gap-2">
            <label class="text-xs font-medium text-gray-700">{{ t("adminBookings.discount") }}</label>
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
              <span class="text-sm text-gray-400">{{ form.discount_mode === "percentage" ? "%" : "₭" }}</span>
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
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-gray-700">{{ t("adminBookings.paymentStatus") }}</label>
            <div class="flex items-center gap-3">
              <input
                v-model="form.payment_status"
                type="checkbox"
                class="toggle toggle-sm"
              />
              <span class="text-sm" :class="form.payment_status ? 'text-gray-800' : 'text-gray-400'">
                {{ form.payment_status ? t("adminBookings.paymentPaid") : t("adminBookings.paymentUnpaid") }}
              </span>
            </div>
          </div>

          <!-- Customer info toggle -->
          <div>
            <button type="button" class="text-xs text-blue-600 hover:text-blue-800" @click="showCustomer = !showCustomer">
              {{ showCustomer ? "▾" : "▸" }} {{ t("adminBookings.addCustomer") }}
            </button>
            <div v-if="showCustomer" class="mt-3 grid grid-cols-2 gap-4">
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-gray-700">{{ t("adminBookings.customerName") }}</label>
                <input v-model="form.customer_name" class="input input-bordered input-sm w-full" />
              </div>
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-gray-700">{{ t("adminBookings.customerPhone") }}</label>
                <input v-model="form.customer_phone" class="input input-bordered input-sm w-full" />
              </div>
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-gray-700">{{ t("adminBookings.customerIdType") }}</label>
                <input v-model="form.customer_id_type" class="input input-bordered input-sm w-full" />
              </div>
              <div class="flex flex-col gap-1.5">
                <label class="text-xs font-medium text-gray-700">{{ t("adminBookings.customerIdNumber") }}</label>
                <input v-model="form.customer_id_number" class="input input-bordered input-sm w-full" />
              </div>
            </div>
          </div>

          <div v-if="errorMessage" class="alert alert-error text-sm">{{ errorMessage }}</div>
        </form>

        <!-- Footer -->
        <div class="flex justify-end gap-3 mt-6 pt-4 border-t border-gray-100">
          <form method="dialog">
            <button class="btn btn-sm btn-outline">{{ t("adminBookings.cancel") }}</button>
          </form>
          <button
            type="submit"
            form="booking-form"
            class="btn btn-sm bg-gray-900 text-white hover:bg-gray-700 border-none"
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
          <h3 class="font-semibold text-gray-900">{{ statPanelTitle }}</h3>
          <form method="dialog">
            <button class="btn btn-sm btn-circle btn-ghost">✕</button>
          </form>
        </div>
        <div class="overflow-y-auto flex-1">
          <div v-if="statPanelBookings.length === 0" class="py-8 text-center text-sm text-gray-400">
            {{ t("adminBookings.noBookings") }}
          </div>
          <div v-else class="flex flex-col gap-2">
            <button
              v-for="b in statPanelBookings"
              :key="b.id"
              class="w-full text-left rounded-lg border border-gray-100 px-4 py-3 hover:bg-gray-50 transition-colors"
              @click="openDetailFromPanel(b.id)"
            >
              <div class="flex items-center justify-between gap-2">
                <span class="font-mono text-xs font-semibold text-gray-500 bg-gray-100 rounded px-1.5 py-0.5">{{ b.booking_ref }}</span>
                <span :class="['badge badge-sm', b.payment_status === 'paid' ? 'badge-success' : 'badge-warning']">
                  {{ b.payment_status === 'paid' ? t('adminBookings.paid') : t('adminBookings.unpaid') }}
                </span>
              </div>
              <p class="mt-1 text-sm font-medium text-gray-800">{{ b.customer_name || "—" }}</p>
              <p class="text-xs text-gray-400 mt-0.5">{{ formatDate(b.check_in) }} → {{ formatDate(b.check_out) }}</p>
            </button>
          </div>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button>close</button></form>
    </dialog>
  </AppShell>
</template>

