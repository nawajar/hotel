<script setup lang="ts">
import AppShell from "@/components/AppShell.vue";
import { useAuthStore } from "@/stores/auth";

const authStore = useAuthStore();
</script>

<template>
  <AppShell>
    <div class="bg-white rounded-lg border border-gray-200 p-6">
      <h1 class="text-lg font-semibold text-gray-900">Dashboard</h1>
      <p class="mt-2 text-sm text-gray-600">
        Welcome, <span class="font-medium text-gray-900">{{ authStore.user?.name }}</span>.
        You're signed in as
        <span class="font-medium text-gray-900">{{ authStore.user?.role }}</span>.
      </p>
    </div>
  </AppShell>
</template>
