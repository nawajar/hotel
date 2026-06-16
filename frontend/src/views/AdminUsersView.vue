<script setup lang="ts">
import { reactive, ref } from "vue";
import { useI18n } from "vue-i18n";
import AppShell from "@/components/AppShell.vue";
import InputText from "primevue/inputtext";
import Password from "primevue/password";
import Button from "primevue/button";
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
          <InputText
            v-model="form.email"
            type="email"
            required
            class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm"
          />
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">{{ t("adminUsers.name") }}</label>
          <InputText
            v-model="form.name"
            required
            class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm"
          />
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">{{ t("adminUsers.password") }}</label>
          <Password
            v-model="form.password"
            :feedback="false"
            toggle-mask
            input-class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm"
            class="w-full"
          />
        </div>
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">{{ t("adminUsers.role") }}</label>
          <select v-model="form.role" class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm">
            <option value="employee">{{ t("adminUsers.roleEmployee") }}</option>
            <option value="admin">{{ t("adminUsers.roleAdmin") }}</option>
          </select>
        </div>
        <div class="sm:col-span-4">
          <Button
            type="submit"
            :label="t('adminUsers.add')"
            :loading="createUserMutation.isPending.value"
            class="rounded-md bg-gray-900 text-white px-4 py-2 text-sm font-medium hover:bg-gray-800"
          />
        </div>
      </form>

      <p v-if="errorMessage" class="mt-3 text-sm text-red-600">{{ errorMessage }}</p>
      <p v-if="successMessage" class="mt-3 text-sm text-green-600">{{ successMessage }}</p>

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
