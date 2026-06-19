<script setup lang="ts">
import { reactive, ref } from "vue";
import { useI18n } from "vue-i18n";
import AppShell from "@/components/AppShell.vue";
import { useUsersQuery, useCreateUserMutation } from "@/composables/useUsersQueries";
import { ApiError } from "@/api/client";

const { t } = useI18n();

const { data: users, isLoading } = useUsersQuery();
const createUserMutation = useCreateUserMutation();

const form = reactive({
  email: "",
  name: "",
  password: "",
  role: "employee" as "admin" | "employee",
});
const showPassword = ref(false);
const errorMessage = ref("");
const successMessage = ref("");

async function handleSubmit() {
  errorMessage.value = "";
  successMessage.value = "";
  try {
    await createUserMutation.mutateAsync({ ...form });
    successMessage.value = t("adminUsers.added");
    form.email = "";
    form.name = "";
    form.password = "";
    form.role = "employee";
  } catch (err) {
    errorMessage.value = err instanceof ApiError ? err.message : t("adminUsers.genericError");
  }
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleString();
}
</script>

<template>
  <AppShell>
    <div class="bg-white rounded-lg border border-gray-200 p-6">
      <h1 class="text-lg font-semibold text-gray-900">{{ t("adminUsers.title") }}</h1>
      <p class="mt-2 text-sm text-gray-600">{{ t("adminUsers.description") }}</p>

      <form class="mt-6 grid grid-cols-1 sm:grid-cols-4 gap-3 items-end" @submit.prevent="handleSubmit">
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">{{ t("adminUsers.email") }}</label>
          <input v-model="form.email" type="email" required class="input input-bordered input-sm w-full" />
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">{{ t("adminUsers.name") }}</label>
          <input v-model="form.name" required class="input input-bordered input-sm w-full" />
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">{{ t("adminUsers.password") }}</label>
          <div class="relative">
            <input
              v-model="form.password"
              :type="showPassword ? 'text' : 'password'"
              required
              class="input input-bordered input-sm w-full pr-9"
            />
            <button
              type="button"
              class="absolute inset-y-0 right-2 flex items-center text-gray-400 hover:text-gray-600"
              @click="showPassword = !showPassword"
            >
              <svg v-if="showPassword" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
                <line x1="1" y1="1" x2="23" y2="23"/>
              </svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
            </button>
          </div>
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">{{ t("adminUsers.role") }}</label>
          <select v-model="form.role" class="select select-bordered select-sm w-full">
            <option value="employee">{{ t("adminUsers.roleEmployee") }}</option>
            <option value="admin">{{ t("adminUsers.roleAdmin") }}</option>
          </select>
        </div>
        <div class="sm:col-span-4">
          <button
            type="submit"
            class="btn btn-sm bg-gray-900 text-white hover:bg-gray-700 border-none"
            :class="createUserMutation.isPending.value ? 'loading' : ''"
            :disabled="createUserMutation.isPending.value"
          >
            {{ t("adminUsers.add") }}
          </button>
        </div>
      </form>

      <div v-if="errorMessage" class="alert alert-error mt-3 text-sm">{{ errorMessage }}</div>
      <div v-if="successMessage" class="alert alert-success mt-3 text-sm">{{ successMessage }}</div>

      <div v-if="isLoading" class="mt-8 text-sm text-gray-500">…</div>
      <table v-else class="mt-8 w-full text-sm">
        <thead>
          <tr class="text-left text-gray-500 border-b border-gray-200">
            <th class="py-2 pr-4 font-medium">{{ t("adminUsers.email") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminUsers.name") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminUsers.role") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminUsers.createdAt") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in users" :key="user.id" class="border-b border-gray-100">
            <td class="py-2 pr-4 text-gray-900">{{ user.email }}</td>
            <td class="py-2 pr-4 text-gray-900">{{ user.name }}</td>
            <td class="py-2 pr-4 text-gray-600">{{ user.role }}</td>
            <td class="py-2 pr-4 text-gray-600">{{ formatDate(user.created_at) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </AppShell>
</template>
