<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { useI18n } from "vue-i18n";
import AppShell from "@/components/AppShell.vue";
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
const savedField = ref<string | null>(null);

function fieldKey(key: string, locale: Locale) {
  return `${key}:${locale}`;
}

function valueFor(key: string, locale: Locale): string {
  const draft = drafts[fieldKey(key, locale)];
  if (draft !== undefined) return draft;
  return overridesMap.value.get(key)?.get(locale) ?? defaults[locale][key] ?? defaults.en[key];
}

function isOverridden(key: string, locale: Locale): boolean {
  return overridesMap.value.get(key)?.has(locale) ?? false;
}

function onInput(key: string, locale: Locale, value: string) {
  drafts[fieldKey(key, locale)] = value;
}

async function save(key: string, locale: Locale) {
  const value = valueFor(key, locale);
  const field = fieldKey(key, locale);
  savingField.value = field;
  try {
    await translationsApi.adminUpsert(key, locale, value);
    setMessageLive(locale, key, value);
    delete drafts[field];
    await queryClient.invalidateQueries({ queryKey: ["admin-translations"] });
    savedField.value = field;
    setTimeout(() => {
      if (savedField.value === field) savedField.value = null;
    }, 2000);
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
        <div class="pb-2 grid grid-cols-[12rem_1fr_1fr] gap-4 text-xs font-medium text-gray-500 uppercase tracking-wide">
          <div>{{ t("adminTranslations.key") }}</div>
          <div>{{ t("adminTranslations.english") }}</div>
          <div>{{ t("adminTranslations.lao") }}</div>
        </div>
        <div v-for="key in allKeys" :key="key" class="py-3 grid grid-cols-[12rem_1fr_1fr] gap-4 items-start">
          <div class="font-mono text-xs text-gray-500 pt-2">{{ key }}</div>

          <div>
            <div class="flex items-center gap-2">
              <input
                :value="valueFor(key, 'en')"
                class="input input-bordered input-xs w-full"
                @input="onInput(key, 'en', ($event.target as HTMLInputElement).value)"
              />
              <button
                class="btn btn-xs bg-gray-900 text-white hover:bg-gray-700 border-none whitespace-nowrap"
                :class="savingField === fieldKey(key, 'en') ? 'loading' : ''"
                :disabled="savingField === fieldKey(key, 'en')"
                @click="save(key, 'en')"
              >{{ t("adminTranslations.save") }}</button>
            </div>
            <span v-if="savedField === fieldKey(key, 'en')" class="mt-1 inline-block text-xs text-green-600">{{ t("adminTranslations.saved") }}</span>
            <button v-else-if="isOverridden(key, 'en')" class="mt-1 text-xs text-gray-400 hover:text-gray-600 underline" @click="reset(key, 'en')">
              {{ t("adminTranslations.reset") }}
            </button>
            <span v-else class="mt-1 inline-block text-xs text-gray-400">{{ t("adminTranslations.default") }}</span>
          </div>

          <div>
            <div class="flex items-center gap-2">
              <input
                :value="valueFor(key, 'lo')"
                class="input input-bordered input-xs w-full"
                @input="onInput(key, 'lo', ($event.target as HTMLInputElement).value)"
              />
              <button
                class="btn btn-xs bg-gray-900 text-white hover:bg-gray-700 border-none whitespace-nowrap"
                :class="savingField === fieldKey(key, 'lo') ? 'loading' : ''"
                :disabled="savingField === fieldKey(key, 'lo')"
                @click="save(key, 'lo')"
              >{{ t("adminTranslations.save") }}</button>
            </div>
            <span v-if="savedField === fieldKey(key, 'lo')" class="mt-1 inline-block text-xs text-green-600">{{ t("adminTranslations.saved") }}</span>
            <button v-else-if="isOverridden(key, 'lo')" class="mt-1 text-xs text-gray-400 hover:text-gray-600 underline" @click="reset(key, 'lo')">
              {{ t("adminTranslations.reset") }}
            </button>
            <span v-else class="mt-1 inline-block text-xs text-gray-400">{{ t("adminTranslations.default") }}</span>
          </div>
        </div>
      </div>
    </div>
  </AppShell>
</template>
