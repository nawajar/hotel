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
import th from "@/locales/th.json";

const { t } = useI18n();
const queryClient = useQueryClient();

const defaults: Record<Locale, Record<string, string>> = {
  en: flattenMessages(en),
  lo: flattenMessages(lo),
  th: flattenMessages(th),
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

const groupedKeys = computed(() => {
  const groups = new Map<string, string[]>();
  for (const key of allKeys) {
    const group = key.split(".")[0];
    if (!groups.has(group)) groups.set(group, []);
    groups.get(group)!.push(key);
  }
  return groups;
});

const activeGroup = ref<string>([...groupedKeys.value.keys()][0] ?? '');

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
      <div v-else class="mt-6">

        <!-- Tab bar -->
        <div class="overflow-x-auto">
          <div class="flex flex-nowrap gap-1 border-b border-gray-200 pb-0">
            <button
              v-for="[group, keys] in groupedKeys"
              :key="group"
              class="flex items-center gap-1.5 px-3 py-2 whitespace-nowrap text-sm rounded-t-md border border-b-0 transition-colors"
              :class="group === activeGroup
                ? 'bg-white border-gray-200 text-gray-900 -mb-px z-10'
                : 'bg-gray-50 border-transparent text-gray-500 hover:text-gray-700 hover:bg-gray-100'"
              @click="activeGroup = group"
            >
              <span class="font-mono text-xs font-semibold">{{ group }}</span>
              <span class="text-xs text-gray-400">{{ keys.length }}</span>
            </button>
          </div>
        </div>

        <!-- Active group content panel -->
        <div
          v-if="groupedKeys.get(activeGroup)"
          class="rounded-b-lg rounded-tr-lg border border-t-0 border-gray-200 overflow-hidden"
        >
          <!-- Column headers -->
          <div class="grid grid-cols-[10rem_1fr_1fr_1fr] gap-4 px-4 py-2 border-b border-gray-100 text-xs font-medium text-gray-400 uppercase tracking-wide bg-white">
            <div>{{ t("adminTranslations.key") }}</div>
            <div>{{ t("adminTranslations.english") }}</div>
            <div>{{ t("adminTranslations.lao") }}</div>
            <div>{{ t("adminTranslations.thai") }}</div>
          </div>

          <!-- Rows -->
          <div
            v-for="key in groupedKeys.get(activeGroup) ?? []"
            :key="key"
            class="grid grid-cols-[10rem_1fr_1fr_1fr] gap-4 px-4 py-3 border-b border-gray-50 last:border-b-0 items-start hover:bg-gray-50/50"
          >
            <div class="font-mono text-xs text-gray-500 pt-2 truncate" :title="key">
              {{ key.split(".").slice(1).join(".") }}
            </div>

            <!-- English -->
            <div>
              <div class="flex items-center gap-2">
                <input
                  :value="valueFor(key, 'en')"
                  class="input input-bordered input-sm w-full text-sm"
                  @input="onInput(key, 'en', ($event.target as HTMLInputElement).value)"
                />
                <button
                  class="btn btn-sm bg-gray-900 text-white hover:bg-gray-700 border-none whitespace-nowrap"
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

            <!-- Lao -->
            <div>
              <div class="flex items-center gap-2">
                <input
                  :value="valueFor(key, 'lo')"
                  class="input input-bordered input-sm w-full text-sm"
                  @input="onInput(key, 'lo', ($event.target as HTMLInputElement).value)"
                />
                <button
                  class="btn btn-sm bg-gray-900 text-white hover:bg-gray-700 border-none whitespace-nowrap"
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

            <!-- Thai -->
            <div>
              <div class="flex items-center gap-2">
                <input
                  :value="valueFor(key, 'th')"
                  class="input input-bordered input-sm w-full text-sm"
                  @input="onInput(key, 'th', ($event.target as HTMLInputElement).value)"
                />
                <button
                  class="btn btn-sm bg-gray-900 text-white hover:bg-gray-700 border-none whitespace-nowrap"
                  :class="savingField === fieldKey(key, 'th') ? 'loading' : ''"
                  :disabled="savingField === fieldKey(key, 'th')"
                  @click="save(key, 'th')"
                >{{ t("adminTranslations.save") }}</button>
              </div>
              <span v-if="savedField === fieldKey(key, 'th')" class="mt-1 inline-block text-xs text-green-600">{{ t("adminTranslations.saved") }}</span>
              <button v-else-if="isOverridden(key, 'th')" class="mt-1 text-xs text-gray-400 hover:text-gray-600 underline" @click="reset(key, 'th')">
                {{ t("adminTranslations.reset") }}
              </button>
              <span v-else class="mt-1 inline-block text-xs text-gray-400">{{ t("adminTranslations.default") }}</span>
            </div>
          </div>
        </div>

      </div>
    </div>
  </AppShell>
</template>
