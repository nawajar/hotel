<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { useI18n } from "vue-i18n";
import AppShell from "@/components/AppShell.vue";
import InputText from "primevue/inputtext";
import Button from "primevue/button";
import { translationsApi } from "@/api/translations";
import { flattenMessages } from "@/i18n/flatten";
import { setMessageLive, type Locale } from "@/i18n";
import en from "@/locales/en.json";
import lo from "@/locales/lo.json";

const { t } = useI18n();
const queryClient = useQueryClient();

const defaults: Record<Locale, Record<string, string>> = {
  en: flattenMessages(en),
  lo: flattenMessages(lo),
};
const allKeys = Object.keys(defaults.en).sort();

const { data: overridesList, isLoading } = useQuery({
  queryKey: ["admin-translations"],
  queryFn: translationsApi.adminList,
});

const overridesMap = computed(() => {
  const map = new Map<string, Map<string, string>>();
  for (const o of overridesList.value ?? []) {
    if (!map.has(o.key)) map.set(o.key, new Map());
    map.get(o.key)!.set(o.locale, o.value);
  }
  return map;
});

const drafts = reactive<Record<string, string>>({});
const savingField = ref<string | null>(null);

function fieldKey(key: string, locale: Locale) {
  return `${key}:${locale}`;
}

function valueFor(key: string, locale: Locale): string {
  const draft = drafts[fieldKey(key, locale)];
  if (draft !== undefined) return draft;
  return overridesMap.value.get(key)?.get(locale) ?? defaults[locale][key];
}

function isOverridden(key: string, locale: Locale): boolean {
  return overridesMap.value.get(key)?.has(locale) ?? false;
}

function onInput(key: string, locale: Locale, value: string) {
  drafts[fieldKey(key, locale)] = value;
}

async function save(key: string, locale: Locale) {
  const value = valueFor(key, locale);
  savingField.value = fieldKey(key, locale);
  try {
    await translationsApi.adminUpsert(key, locale, value);
    setMessageLive(locale, key, value);
    delete drafts[fieldKey(key, locale)];
    await queryClient.invalidateQueries({ queryKey: ["admin-translations"] });
  } finally {
    savingField.value = null;
  }
}

async function reset(key: string, locale: Locale) {
  savingField.value = fieldKey(key, locale);
  try {
    await translationsApi.adminDelete(key, locale);
    setMessageLive(locale, key, defaults[locale][key]);
    delete drafts[fieldKey(key, locale)];
    await queryClient.invalidateQueries({ queryKey: ["admin-translations"] });
  } finally {
    savingField.value = null;
  }
}
</script>

<template>
  <AppShell>
    <div class="bg-white rounded-lg border border-gray-200 p-6">
      <h1 class="text-lg font-semibold text-gray-900">{{ t("adminTranslations.title") }}</h1>
      <p class="mt-2 text-sm text-gray-600">{{ t("adminTranslations.description") }}</p>

      <div v-if="isLoading" class="mt-6 text-sm text-gray-500">…</div>
      <div v-else class="mt-6 divide-y divide-gray-100">
        <div v-for="key in allKeys" :key="key" class="py-3 grid grid-cols-[12rem_1fr_1fr] gap-4 items-start">
          <div class="font-mono text-xs text-gray-500 pt-2">{{ key }}</div>

          <div>
            <div class="flex items-center gap-2">
              <InputText
                :model-value="valueFor(key, 'en')"
                class="w-full rounded-md border border-gray-300 px-2 py-1.5 text-sm"
                @update:model-value="(v) => onInput(key, 'en', String(v))"
              />
              <Button
                :label="t('adminTranslations.save')"
                :loading="savingField === fieldKey(key, 'en')"
                class="text-xs px-2 py-1.5 rounded-md bg-gray-900 text-white whitespace-nowrap"
                @click="save(key, 'en')"
              />
            </div>
            <button
              v-if="isOverridden(key, 'en')"
              class="mt-1 text-xs text-gray-400 hover:text-gray-600 underline"
              @click="reset(key, 'en')"
            >
              {{ t("adminTranslations.reset") }}
            </button>
            <span v-else class="mt-1 inline-block text-xs text-gray-400">{{
              t("adminTranslations.default")
            }}</span>
          </div>

          <div>
            <div class="flex items-center gap-2">
              <InputText
                :model-value="valueFor(key, 'lo')"
                class="w-full rounded-md border border-gray-300 px-2 py-1.5 text-sm"
                @update:model-value="(v) => onInput(key, 'lo', String(v))"
              />
              <Button
                :label="t('adminTranslations.save')"
                :loading="savingField === fieldKey(key, 'lo')"
                class="text-xs px-2 py-1.5 rounded-md bg-gray-900 text-white whitespace-nowrap"
                @click="save(key, 'lo')"
              />
            </div>
            <button
              v-if="isOverridden(key, 'lo')"
              class="mt-1 text-xs text-gray-400 hover:text-gray-600 underline"
              @click="reset(key, 'lo')"
            >
              {{ t("adminTranslations.reset") }}
            </button>
            <span v-else class="mt-1 inline-block text-xs text-gray-400">{{
              t("adminTranslations.default")
            }}</span>
          </div>
        </div>
      </div>
    </div>
  </AppShell>
</template>
