<script setup lang="ts">
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useAuthStore } from "@/stores/auth";
import { setLocale, type Locale } from "@/i18n";
import { ref, onMounted, onUnmounted } from "vue";

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
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <nav class="bg-white border-b border-gray-200">
      <div class="max-w-5xl mx-auto px-4 flex items-center justify-between h-14">
        <div class="flex items-center gap-6">
          <span class="font-semibold text-gray-900">{{ t("nav.brand") }}</span>
          <router-link
            to="/dashboard"
            class="text-sm text-gray-600 hover:text-gray-900"
            active-class="text-gray-900 font-medium"
          >
            {{ t("nav.dashboard") }}
          </router-link>
          <router-link
            v-if="authStore.isAdmin"
            to="/admin"
            class="text-sm text-gray-600 hover:text-gray-900"
            active-class="text-gray-900 font-medium"
          >
            {{ t("nav.admin") }}
          </router-link>
          <router-link
            v-if="authStore.isAdmin"
            to="/admin/translations"
            class="text-sm text-gray-600 hover:text-gray-900"
            active-class="text-gray-900 font-medium"
          >
            {{ t("nav.translations") }}
          </router-link>
          <router-link
            v-if="authStore.isAdmin"
            to="/admin/users"
            class="text-sm text-gray-600 hover:text-gray-900"
            active-class="text-gray-900 font-medium"
          >
            {{ t("nav.users") }}
          </router-link>
          <router-link
            v-if="authStore.isAdmin"
            to="/admin/rooms"
            class="text-sm text-gray-600 hover:text-gray-900"
            active-class="text-gray-900 font-medium"
          >
            {{ t("nav.rooms") }}
          </router-link>
          <router-link
            to="/admin/bookings"
            class="text-sm text-gray-600 hover:text-gray-900"
            active-class="text-gray-900 font-medium"
          >
            {{ t("nav.bookings") }}
          </router-link>
          <router-link
            to="/calendar"
            class="text-sm text-gray-600 hover:text-gray-900"
            active-class="text-gray-900 font-medium"
          >
            {{ t("nav.calendar") }}
          </router-link>
          <router-link
            v-if="authStore.isAdmin"
            to="/admin/income-summary"
            class="text-sm text-gray-600 hover:text-gray-900"
            active-class="text-gray-900 font-medium"
          >
            {{ t("nav.incomeSummary") }}
          </router-link>
        </div>
        <div class="flex items-center gap-4">
          <span class="text-sm text-gray-600 font-mono">{{ currentTime }}</span>
          <select
            :value="locale"
            class="text-sm border border-gray-300 rounded-md px-2 py-1 text-gray-700"
            @change="handleLocaleChange"
          >
            <option value="en">EN</option>
            <option value="lo">ລາວ</option>
          </select>
          <span class="text-sm text-gray-500">{{ authStore.user?.name }}</span>
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
