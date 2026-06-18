<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import FullCalendar from "@fullcalendar/vue3";
import type { CalendarOptions, EventInput, EventClickArg, DatesSetArg } from "@fullcalendar/core";
import type { ResourceInput } from "@fullcalendar/resource";
import resourceTimelinePlugin from "@fullcalendar/resource-timeline";
import dayGridPlugin from "@fullcalendar/daygrid";
import interactionPlugin from "@fullcalendar/interaction";
import AppShell from "@/components/AppShell.vue";
import { useAuthStore } from "@/stores/auth";
import { useCalendarBookingsQuery, useCalendarRoomsQuery } from "@/composables/useCalendarQueries";
import type { CalendarBooking } from "@/api/calendar";

const { t } = useI18n();
const router = useRouter();
const authStore = useAuthStore();

// ---- helpers ----
function todayMonthStart(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-01`;
}
function todayMonthEnd(): string {
  const d = new Date();
  const last = new Date(d.getFullYear(), d.getMonth() + 1, 0);
  return `${last.getFullYear()}-${String(last.getMonth() + 1).padStart(2, "0")}-${String(last.getDate()).padStart(2, "0")}`;
}

// ---- state ----
const isMobile = ref(false);
const currentView = ref<"resourceTimelineMonth" | "dayGridMonth">("dayGridMonth");
const rangeStart = ref(todayMonthStart());
const rangeEnd = ref(todayMonthEnd());
const selectedBooking = ref<CalendarBooking | null>(null);
const calendarRef = ref<InstanceType<typeof FullCalendar> | null>(null);
const currentTitle = ref(
  new Date().toLocaleDateString(undefined, { month: "long", year: "numeric" })
);

// ---- queries ----
const { data: rooms, isLoading: roomsLoading } = useCalendarRoomsQuery();
const { data: bookings, isLoading: bookingsLoading, isFetching: bookingsFetching } = useCalendarBookingsQuery(rangeStart, rangeEnd);

// true only on first load (no data yet) — navigating months uses isFetching instead
const isInitialLoading = computed(() => roomsLoading.value || bookingsLoading.value);
const isFetching = computed(() => bookingsFetching.value);

// ---- color logic ----
function colorForBooking(b: CalendarBooking): string {
  if (b.label === "needs_attention") return "#ef4444";
  if (b.label === "check_in") return "#3b82f6";
  if (b.label === "check_out") return "#a855f7";
  if (b.status === "cancelled") return "#9ca3af";
  if (b.payment_status === "paid") return "#22c55e";
  return "#f59e0b";
}

// ---- events ----
const events = computed<EventInput[]>(() => {
  if (!bookings.value) return [];
  return bookings.value.map((b) => {
    const color = colorForBooking(b);
    return {
      id: b.id,
      title: b.booking_ref + (b.customer_name ? ` – ${b.customer_name}` : ""),
      start: b.check_in.slice(0, 10),
      end: b.check_out.slice(0, 10),
      resourceIds: b.rooms.map((r) => r.id),
      backgroundColor: color,
      borderColor: color,
      extendedProps: { booking: b },
    };
  });
});

// ---- resources ----
const resources = computed<ResourceInput[]>(() => {
  if (!rooms.value) return [];
  return rooms.value.map((r) => ({
    id: r.id,
    title: `${r.room_number} – ${r.room_name}`,
  }));
});

// ---- event handlers ----
function handleEventClick(info: EventClickArg) {
  selectedBooking.value = info.event.extendedProps.booking as CalendarBooking;
}

function handleDatesSet(arg: DatesSetArg) {
  rangeStart.value = arg.startStr.slice(0, 10);
  rangeEnd.value = arg.endStr.slice(0, 10);
  currentTitle.value = arg.view.title;
}

// ---- calendar options ----
const calendarOptions = computed<CalendarOptions>(() => ({
  plugins: [resourceTimelinePlugin, dayGridPlugin, interactionPlugin],
  schedulerLicenseKey: "GPL-My-Project-Is-Open-Source",
  initialView: isMobile.value ? "dayGridMonth" : currentView.value,
  headerToolbar: false,
  resources: resources.value,
  events: events.value,
  eventClick: handleEventClick,
  datesSet: handleDatesSet,
  height: "auto",
  resourceAreaHeaderContent: t("calendar.rooms"),
}));

// ---- toolbar actions ----
function calApi() {
  return calendarRef.value?.getApi();
}

function goPrev() {
  calApi()?.prev();
}
function goToday() {
  calApi()?.today();
}
function goNext() {
  calApi()?.next();
}

function switchView(view: "resourceTimelineMonth" | "dayGridMonth") {
  currentView.value = view;
  calApi()?.changeView(view);
}

// ---- detail dialog ----
const showDetail = computed({
  get: () => !!selectedBooking.value,
  set: (v: boolean) => {
    if (!v) selectedBooking.value = null;
  },
});

function closeDetail() {
  selectedBooking.value = null;
}

function goEditBooking() {
  router.push({ name: "admin-bookings", query: { id: selectedBooking.value?.id } });
}

// ---- mobile detection ----
function checkMobile() {
  const mobile = window.innerWidth < 768;
  if (mobile !== isMobile.value) {
    isMobile.value = mobile;
    if (mobile) {
      calApi()?.changeView("dayGridMonth");
    } else {
      calApi()?.changeView(currentView.value);
    }
  }
}

onMounted(() => {
  isMobile.value = window.innerWidth < 768;
  window.addEventListener("resize", checkMobile);
});

onUnmounted(() => {
  window.removeEventListener("resize", checkMobile);
});

// ---- watch for mobile flip before calendar renders (initial) ----
watch(isMobile, (mobile) => {
  if (mobile) {
    calApi()?.changeView("dayGridMonth");
  } else {
    calApi()?.changeView(currentView.value);
  }
});

// ---- legend entries ----
const legendEntries = computed(() => [
  { color: "#ef4444", label: t("calendar.legend.needsAttention") },
  { color: "#3b82f6", label: t("calendar.legend.checkIn") },
  { color: "#a855f7", label: t("calendar.legend.checkOut") },
  { color: "#22c55e", label: t("calendar.legend.paid") },
  { color: "#f59e0b", label: t("calendar.legend.unpaid") },
  { color: "#9ca3af", label: t("calendar.legend.cancelled") },
]);
</script>

<template>
  <AppShell>
    <div class="space-y-4">
      <!-- Page title -->
      <div class="flex items-center justify-between">
        <h1 class="text-lg font-semibold text-gray-900">{{ t("calendar.title") }}</h1>
      </div>

      <!-- Calendar card -->
      <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
        <!-- Custom toolbar -->
        <div class="flex items-center gap-2 px-4 py-3 border-b border-gray-200 flex-wrap">
          <button
            class="text-sm px-3 py-1.5 rounded-md border border-gray-300 text-gray-700 hover:bg-gray-100"
            @click="goPrev"
          >
            &lsaquo;
          </button>
          <span class="min-w-[10rem] text-center text-sm font-semibold text-gray-900 select-none flex items-center justify-center gap-2">
            {{ currentTitle }}
            <svg v-if="isFetching" class="animate-spin h-3.5 w-3.5 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
            </svg>
          </span>
          <button
            class="text-sm px-3 py-1.5 rounded-md border border-gray-300 text-gray-700 hover:bg-gray-100"
            @click="goNext"
          >
            &rsaquo;
          </button>
          <button
            class="text-sm px-3 py-1.5 rounded-md border border-gray-300 text-gray-700 hover:bg-gray-100"
            @click="goToday"
          >
            {{ t("calendar.today") }}
          </button>

          <div v-if="!isMobile" class="ml-4 flex items-center gap-2">
            <button
              :class="[
                'text-sm px-3 py-1.5 rounded-md border',
                currentView === 'resourceTimelineMonth'
                  ? 'bg-gray-900 text-white border-gray-900'
                  : 'border-gray-300 text-gray-700 hover:bg-gray-100',
              ]"
              @click="switchView('resourceTimelineMonth')"
            >
              {{ t("calendar.timeline") }}
            </button>
            <button
              :class="[
                'text-sm px-3 py-1.5 rounded-md border',
                currentView === 'dayGridMonth'
                  ? 'bg-gray-900 text-white border-gray-900'
                  : 'border-gray-300 text-gray-700 hover:bg-gray-100',
              ]"
              @click="switchView('dayGridMonth')"
            >
              {{ t("calendar.month") }}
            </button>
          </div>
        </div>

        <!-- Skeleton: only on very first load; placeholderData keeps isInitialLoading false on navigation -->
        <div v-if="isInitialLoading" class="animate-pulse p-6 space-y-3">
          <div class="h-8 bg-gray-200 rounded w-1/3"></div>
          <div class="h-64 bg-gray-100 rounded"></div>
          <div class="h-6 bg-gray-200 rounded w-1/2"></div>
          <div class="h-6 bg-gray-200 rounded w-2/3"></div>
          <div class="h-6 bg-gray-200 rounded w-1/2"></div>
        </div>

        <!-- Calendar: mounts into a visible container so FullCalendar sizes correctly -->
        <div v-else class="p-2">
          <FullCalendar ref="calendarRef" :options="calendarOptions" />
        </div>
      </div>

      <!-- Legend -->
      <div class="flex flex-wrap gap-3">
        <div
          v-for="entry in legendEntries"
          :key="entry.label"
          class="flex items-center gap-1.5"
        >
          <span
            class="inline-block w-3 h-3 rounded-full"
            :style="{ backgroundColor: entry.color }"
          ></span>
          <span class="text-xs text-gray-600">{{ entry.label }}</span>
        </div>
      </div>
    </div>

    <!-- Detail dialog -->
    <div
      v-if="showDetail && selectedBooking"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      @click.self="closeDetail"
    >
      <div class="bg-white rounded-lg shadow-xl w-full max-w-md overflow-hidden">
        <!-- Dialog header -->
        <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200">
          <h2 class="text-base font-semibold text-gray-900">
            {{ selectedBooking.booking_ref }}
          </h2>
          <button
            class="text-gray-400 hover:text-gray-600 text-xl leading-none"
            @click="closeDetail"
          >
            &times;
          </button>
        </div>

        <!-- Dialog body -->
        <div class="px-6 py-4 space-y-3 text-sm text-gray-700">
          <div v-if="selectedBooking.customer_name" class="flex gap-2">
            <span class="text-gray-500 w-28 shrink-0">{{ t("calendar.customer") }}</span>
            <span class="font-medium">{{ selectedBooking.customer_name }}</span>
          </div>
          <div class="flex gap-2">
            <span class="text-gray-500 w-28 shrink-0">{{ t("calendar.checkIn") }}</span>
            <span>{{ selectedBooking.check_in.slice(0, 10) }}</span>
          </div>
          <div class="flex gap-2">
            <span class="text-gray-500 w-28 shrink-0">{{ t("calendar.checkOut") }}</span>
            <span>{{ selectedBooking.check_out.slice(0, 10) }}</span>
          </div>
          <div class="flex gap-2">
            <span class="text-gray-500 w-28 shrink-0">{{ t("calendar.status") }}</span>
            <span>{{ selectedBooking.status }}</span>
          </div>
          <div class="flex gap-2">
            <span class="text-gray-500 w-28 shrink-0">{{ t("calendar.paymentStatus") }}</span>
            <span>{{ selectedBooking.payment_status }}</span>
          </div>
          <div v-if="selectedBooking.label" class="flex gap-2">
            <span class="text-gray-500 w-28 shrink-0">{{ t("calendar.label") }}</span>
            <span>{{ selectedBooking.label }}</span>
          </div>
          <div v-if="selectedBooking.note" class="flex gap-2">
            <span class="text-gray-500 w-28 shrink-0">{{ t("calendar.note") }}</span>
            <span>{{ selectedBooking.note }}</span>
          </div>
          <!-- Rooms -->
          <div v-if="selectedBooking.rooms.length > 0" class="flex gap-2">
            <span class="text-gray-500 w-28 shrink-0">{{ t("calendar.rooms") }}</span>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="room in selectedBooking.rooms"
                :key="room.id"
                class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800"
              >
                {{ room.room_number }} – {{ room.room_name }}
              </span>
            </div>
          </div>
        </div>

        <!-- Dialog footer -->
        <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200">
          <button
            v-if="authStore.isAdmin"
            class="text-sm px-3 py-1.5 rounded-md bg-gray-900 text-white hover:bg-gray-800"
            @click="goEditBooking"
          >
            {{ t("calendar.edit") }}
          </button>
          <button
            class="text-sm px-3 py-1.5 rounded-md border border-gray-300 text-gray-700 hover:bg-gray-100"
            @click="closeDetail"
          >
            {{ t("calendar.close") }}
          </button>
        </div>
      </div>
    </div>
  </AppShell>
</template>

<style scoped>
/* Ensure FullCalendar fits the light theme */
:deep(.fc) {
  font-family: inherit;
  font-size: 0.8125rem;
}
:deep(.fc-toolbar-title) {
  font-size: 1rem;
  font-weight: 600;
}
:deep(.fc-event) {
  cursor: pointer;
  border-radius: 3px;
}
:deep(.fc-daygrid-event) {
  white-space: normal;
}
</style>
