<script setup lang="ts">
import { onMounted } from "vue";
import { useSettingsStore } from "@/stores/settings";

const settingsStore = useSettingsStore();
onMounted(() => {
  settingsStore.load();
});
</script>

<template>
  <router-view />
</template>
