<script setup lang="ts">
import { ref } from "vue";
import AppShell from "@/components/AppShell.vue";
import Button from "primevue/button";
import { authApi } from "@/api/auth";
import { ApiError } from "@/api/client";

const pingResult = ref("");

async function handlePing() {
  pingResult.value = "";
  try {
    const res = await authApi.adminPing();
    pingResult.value = `ok: ${res.ok}`;
  } catch (err) {
    pingResult.value = err instanceof ApiError ? `error ${err.status}: ${err.message}` : "error";
  }
}
</script>

<template>
  <AppShell>
    <div class="bg-white rounded-lg border border-gray-200 p-6">
      <h1 class="text-lg font-semibold text-gray-900">Admin</h1>
      <p class="mt-2 text-sm text-gray-600">Admin-only area.</p>

      <Button
        label="Ping /api/admin/ping"
        class="mt-4 rounded-md bg-gray-900 text-white px-4 py-2 text-sm font-medium hover:bg-gray-800"
        @click="handlePing"
      />
      <p v-if="pingResult" class="mt-3 text-sm text-gray-700">{{ pingResult }}</p>
    </div>
  </AppShell>
</template>
