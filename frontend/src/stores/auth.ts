import { defineStore } from "pinia";
import { authApi, type LoginInput, type UserPublic } from "@/api/auth";
import { ApiError } from "@/api/client";

export const useAuthStore = defineStore("auth", {
  state: () => ({
    user: null as UserPublic | null,
    ready: false,
  }),
  getters: {
    isAuthenticated: (state) => state.user !== null,
    isAdmin: (state) => state.user?.role === "admin",
  },
  actions: {
    async init() {
      try {
        this.user = await authApi.me();
      } catch (err) {
        if (!(err instanceof ApiError && err.status === 401)) throw err;
        this.user = null;
      } finally {
        this.ready = true;
      }
    },
    async login(input: LoginInput) {
      this.user = await authApi.login(input);
    },
    async logout() {
      await authApi.logout();
      this.user = null;
    },
  },
});
