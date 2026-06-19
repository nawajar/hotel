<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import AppShell from "@/components/AppShell.vue";
import { authApi } from "@/api/auth";
import { ApiError } from "@/api/client";

const { t } = useI18n();
const pingResult = ref("");

async function handlePing() {
  pingResult.value = "";
  try {
    const res = await authApi.adminPing();
    pingResult.value = t("admin.pingOk", { ok: res.ok });
  } catch (err) {
    pingResult.value =
      err instanceof ApiError ? t("admin.pingError", { status: err.status, message: err.message }) : "error";
  }
}
</script>

<template>
  <AppShell>
    <div class="bg-white rounded-lg border border-gray-200 p-6">
      <h1 class="text-lg font-semibold text-gray-900">{{ t("admin.title") }}</h1>
      <p class="mt-2 text-sm text-gray-600">{{ t("admin.description") }}</p>

      <button class="btn btn-sm mt-4 bg-gray-900 text-white hover:bg-gray-700 border-none" @click="handlePing">
        {{ t("admin.pingButton") }}
      </button>
      <p v-if="pingResult" class="mt-3 text-sm text-gray-700">{{ pingResult }}</p>
    </div>
  </AppShell>
</template>
