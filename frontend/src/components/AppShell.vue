<script setup lang="ts">
import { useRouter } from "vue-router";
import { useAuthStore } from "@/stores/auth";

const router = useRouter();
const authStore = useAuthStore();

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
          <span class="font-semibold text-gray-900">Hotel</span>
          <router-link
            to="/dashboard"
            class="text-sm text-gray-600 hover:text-gray-900"
            active-class="text-gray-900 font-medium"
          >
            Dashboard
          </router-link>
          <router-link
            v-if="authStore.isAdmin"
            to="/admin"
            class="text-sm text-gray-600 hover:text-gray-900"
            active-class="text-gray-900 font-medium"
          >
            Admin
          </router-link>
        </div>
        <div class="flex items-center gap-4">
          <span class="text-sm text-gray-500">{{ authStore.user?.name }}</span>
          <button
            class="text-sm px-3 py-1.5 rounded-md border border-gray-300 text-gray-700 hover:bg-gray-100"
            @click="handleLogout"
          >
            Logout
          </button>
        </div>
      </div>
    </nav>
    <main class="max-w-5xl mx-auto px-4 py-8">
      <slot />
    </main>
  </div>
</template>
