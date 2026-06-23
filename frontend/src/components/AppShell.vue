<script setup lang="ts">
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useAuthStore } from "@/stores/auth";
import { setLocale, type Locale } from "@/i18n";
import { ref, onMounted, onUnmounted, computed } from "vue";

const router = useRouter();
const { t, locale } = useI18n();
const authStore = useAuthStore();

const currentTime = ref("");
let clockTimer: ReturnType<typeof setInterval>;

function updateClock() {
  currentTime.value = new Date().toLocaleTimeString();
}

onMounted(() => {
  updateClock();
  clockTimer = setInterval(updateClock, 1000);
});

onUnmounted(() => {
  clearInterval(clockTimer);
});

function handleLocaleChange(event: Event) {
  setLocale((event.target as HTMLSelectElement).value as Locale);
}

async function handleLogout() {
  await authStore.logout();
  router.push({ name: "login" });
}

const navItems = computed(() => [
  {
    to: "/dashboard",
    label: t("nav.dashboard"),
    adminOnly: false,
    icon: `<path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />`,
  },
  {
    to: "/admin",
    label: t("nav.admin"),
    adminOnly: true,
    icon: `<path stroke-linecap="round" stroke-linejoin="round" d="M9 12h3.75M9 15h3.75M9 18h3.75m3 .75H18a2.25 2.25 0 002.25-2.25V6.108c0-1.135-.845-2.098-1.976-2.192a48.424 48.424 0 00-1.123-.08m-5.801 0c-.065.21-.1.433-.1.664 0 .414.336.75.75.75h4.5a.75.75 0 00.75-.75 2.25 2.25 0 00-.1-.664m-5.8 0A2.251 2.251 0 0113.5 2.25H15c1.012 0 1.867.668 2.15 1.586m-5.8 0c-.376.023-.75.05-1.124.08C9.095 4.01 8.25 4.973 8.25 6.108V8.25m0 0H4.875c-.621 0-1.125.504-1.125 1.125v11.25c0 .621.504 1.125 1.125 1.125h9.75c.621 0 1.125-.504 1.125-1.125V9.375c0-.621-.504-1.125-1.125-1.125H8.25zM6.75 12h.008v.008H6.75V12zm0 3h.008v.008H6.75V15zm0 3h.008v.008H6.75V18z" />`,
  },
  {
    to: "/admin/bookings",
    label: t("nav.bookings"),
    adminOnly: false,
    icon: `<path stroke-linecap="round" stroke-linejoin="round" d="M6.75 3v2.25M17.25 3v2.25M3 18.75V7.5a2.25 2.25 0 012.25-2.25h13.5A2.25 2.25 0 0121 7.5v11.25m-18 0A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75m-18 0v-7.5A2.25 2.25 0 015.25 9h13.5A2.25 2.25 0 0121 11.25v7.5m-9-6h.008v.008H12v-.008zM12 15h.008v.008H12V15zm0 2.25h.008v.008H12v-.008zM9.75 15h.008v.008H9.75V15zm0 2.25h.008v.008H9.75v-.008zM7.5 15h.008v.008H7.5V15zm0 2.25h.008v.008H7.5v-.008zm6.75-4.5h.008v.008h-.008v-.008zm0 2.25h.008v.008h-.008V15zm0 2.25h.008v.008h-.008v-.008zm2.25-4.5h.008v.008H16.5v-.008zm0 2.25h.008v.008H16.5V15z" />`,
  },
  {
    to: "/calendar",
    label: t("nav.calendar"),
    adminOnly: false,
    icon: `<path stroke-linecap="round" stroke-linejoin="round" d="M6.75 3v2.25M17.25 3v2.25M3 18.75V7.5a2.25 2.25 0 012.25-2.25h13.5A2.25 2.25 0 0121 7.5v11.25m-18 0A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75m-18 0v-7.5A2.25 2.25 0 015.25 9h13.5A2.25 2.25 0 0121 11.25v7.5M12 10.5h.008v.008H12V10.5z" />`,
  },
  {
    to: "/admin/rooms",
    label: t("nav.rooms"),
    adminOnly: true,
    icon: `<path stroke-linecap="round" stroke-linejoin="round" d="M3.75 21h16.5M4.5 3h15M5.25 3v18m13.5-18v18M9 6.75h1.5m-1.5 3h1.5m-1.5 3h1.5m3-6H15m-1.5 3H15m-1.5 3H15M9 21v-3.375c0-.621.504-1.125 1.125-1.125h3.75c.621 0 1.125.504 1.125 1.125V21" />`,
  },
  {
    to: "/admin/users",
    label: t("nav.users"),
    adminOnly: true,
    icon: `<path stroke-linecap="round" stroke-linejoin="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z" />`,
  },
  {
    to: "/admin/daily-report",
    label: t("nav.dailyReport"),
    adminOnly: true,
    icon: `<path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />`,
  },
  {
    to: "/admin/income-summary",
    label: t("nav.incomeSummary"),
    adminOnly: true,
    icon: `<path stroke-linecap="round" stroke-linejoin="round" d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z" />`,
  },
  {
    to: "/admin/translations",
    label: t("nav.translations"),
    adminOnly: true,
    icon: `<path stroke-linecap="round" stroke-linejoin="round" d="M10.5 21l5.25-11.25L21 21m-9-3h7.5M3 5.621a48.474 48.474 0 016-.371m0 0c1.12 0 2.233.038 3.334.114M9 5.25V3m3.334 2.364C11.176 10.658 7.69 15.08 3 17.502m9.334-12.138c.896.061 1.785.147 2.666.257m-4.589 8.495a18.023 18.023 0 01-3.827-5.802" />`,
  },
]);

const visibleNavItems = computed(() =>
  navItems.value.filter((item) => !item.adminOnly || authStore.isAdmin)
);
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <nav class="bg-white border-b border-gray-200">
      <div class="max-w-5xl mx-auto px-4 flex items-center justify-between h-14">
        <!-- Left: brand + nav icons -->
        <div class="flex items-center gap-1">
          <span class="font-semibold text-gray-900 mr-3 shrink-0">{{ t("nav.brand") }}</span>

          <div
            v-for="item in visibleNavItems"
            :key="item.to"
            class="relative group"
          >
            <router-link
              :to="item.to"
              class="flex items-center justify-center w-9 h-9 rounded-md text-gray-400 hover:text-gray-900 hover:bg-gray-100 transition-colors"
              active-class="!text-gray-900 bg-gray-100"
              :exact="item.to === '/admin'"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.75"
                v-html="item.icon"
              />
            </router-link>

            <!-- Tooltip -->
            <span
              class="pointer-events-none absolute left-1/2 top-full mt-1.5 -translate-x-1/2 whitespace-nowrap rounded bg-gray-900 px-2 py-1 text-xs text-white opacity-0 transition-opacity group-hover:opacity-100 z-50"
            >
              {{ item.label }}
            </span>
          </div>
        </div>

        <!-- Right: clock, locale, user, logout -->
        <div class="flex items-center gap-4">
          <span class="text-sm text-gray-600 font-mono shrink-0">{{ currentTime }}</span>
          <select
            :value="locale"
            class="text-sm border border-gray-300 rounded-md px-2 py-1 text-gray-700"
            @change="handleLocaleChange"
          >
            <option value="en">EN</option>
            <option value="lo">ລາວ</option>
            <option value="th">ไทย</option>
          </select>
          <span class="text-sm text-gray-500 shrink-0">{{ authStore.user?.name }}</span>
          <button
            class="text-sm px-3 py-1.5 rounded-md border border-gray-300 text-gray-700 hover:bg-gray-100"
            @click="handleLogout"
          >
            {{ t("nav.logout") }}
          </button>
        </div>
      </div>
    </nav>
    <main class="max-w-5xl mx-auto px-4 py-8">
      <slot />
    </main>
  </div>
</template>
