<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useLoginMutation } from "@/composables/useAuthQueries";
import { ApiError } from "@/api/client";

const router = useRouter();
const { t } = useI18n();
const email = ref("");
const password = ref("");
const showPassword = ref(false);
const errorMessage = ref("");

const loginMutation = useLoginMutation();

async function handleSubmit() {
  errorMessage.value = "";
  try {
    await loginMutation.mutateAsync({ email: email.value, password: password.value });
    router.push({ name: "dashboard" });
  } catch (err) {
    errorMessage.value = err instanceof ApiError ? err.message : t("login.genericError");
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-50">
    <form
      class="w-full max-w-sm bg-white p-8 rounded-lg border border-gray-200 shadow-sm"
      @submit.prevent="handleSubmit"
    >
      <h1 class="text-xl font-semibold text-gray-900 mb-6">{{ t("login.title") }}</h1>

      <div class="mb-4">
        <label for="email" class="block text-sm font-medium text-gray-700 mb-1">{{ t("login.email") }}</label>
        <input
          id="email"
          v-model="email"
          type="email"
          required
          class="input input-bordered w-full text-sm"
        />
      </div>

      <div class="mb-6">
        <label for="password" class="block text-sm font-medium text-gray-700 mb-1">{{ t("login.password") }}</label>
        <div class="relative">
          <input
            id="password"
            v-model="password"
            :type="showPassword ? 'text' : 'password'"
            required
            class="input input-bordered w-full text-sm pr-10"
          />
          <button
            type="button"
            class="absolute inset-y-0 right-3 flex items-center text-gray-400 hover:text-gray-600"
            @click="showPassword = !showPassword"
          >
            <svg v-if="showPassword" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
              <line x1="1" y1="1" x2="23" y2="23"/>
            </svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
              <circle cx="12" cy="12" r="3"/>
            </svg>
          </button>
        </div>
      </div>

      <div v-if="errorMessage" class="alert alert-error mb-4 text-sm">{{ errorMessage }}</div>

      <button
        type="submit"
        class="btn w-full bg-gray-900 text-white hover:bg-gray-700 border-none"
        :class="loginMutation.isPending.value ? 'loading' : ''"
        :disabled="loginMutation.isPending.value"
      >
        {{ t("login.submit") }}
      </button>
    </form>
  </div>
</template>
