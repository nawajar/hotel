<script setup lang="ts">
import { useI18n } from "vue-i18n";
import AppShell from "@/components/AppShell.vue";
import { useAuthStore } from "@/stores/auth";

const { t } = useI18n();
const authStore = useAuthStore();
</script>

<template>
  <AppShell>
    <div class="bg-white rounded-lg border border-gray-200 p-6">
      <h1 class="text-lg font-semibold text-gray-900">{{ t("dashboard.title") }}</h1>
      <p class="mt-2 text-sm text-gray-600">
        <i18n-t keypath="dashboard.welcome" tag="span">
          <template #name
            ><span class="font-medium text-gray-900">{{ authStore.user?.name }}</span></template
          >
        </i18n-t>
        <i18n-t keypath="dashboard.signedInAs" tag="span">
          <template #role
            ><span class="font-medium text-gray-900">{{ authStore.user?.role }}</span></template
          >
        </i18n-t>
      </p>
    </div>
  </AppShell>
</template>
