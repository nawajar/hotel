<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import InputText from "primevue/inputtext";
import Password from "primevue/password";
import Button from "primevue/button";
import { useLoginMutation } from "@/composables/useAuthQueries";
import { ApiError } from "@/api/client";

const router = useRouter();
const email = ref("");
const password = ref("");
const errorMessage = ref("");

const loginMutation = useLoginMutation();

async function handleSubmit() {
  errorMessage.value = "";
  try {
    await loginMutation.mutateAsync({ email: email.value, password: password.value });
    router.push({ name: "dashboard" });
  } catch (err) {
    errorMessage.value =
      err instanceof ApiError ? err.message : "Something went wrong. Please try again.";
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-50">
    <form
      class="w-full max-w-sm bg-white p-8 rounded-lg border border-gray-200 shadow-sm"
      @submit.prevent="handleSubmit"
    >
      <h1 class="text-xl font-semibold text-gray-900 mb-6">Sign in</h1>

      <div class="mb-4">
        <label for="email" class="block text-sm font-medium text-gray-700 mb-1">Email</label>
        <InputText
          id="email"
          v-model="email"
          type="email"
          required
          class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400"
        />
      </div>

      <div class="mb-6">
        <label for="password" class="block text-sm font-medium text-gray-700 mb-1">Password</label>
        <Password
          id="password"
          v-model="password"
          :feedback="false"
          toggle-mask
          input-class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400"
          class="w-full"
        />
      </div>

      <p v-if="errorMessage" class="mb-4 text-sm text-red-600">{{ errorMessage }}</p>

      <Button
        type="submit"
        label="Sign in"
        :loading="loginMutation.isPending.value"
        class="w-full rounded-md bg-gray-900 text-white py-2 text-sm font-medium hover:bg-gray-800 disabled:opacity-50"
      />
    </form>
  </div>
</template>
