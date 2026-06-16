import { createApp } from "vue";
import { createPinia } from "pinia";
import { VueQueryPlugin } from "@tanstack/vue-query";
import PrimeVue from "primevue/config";

import App from "./App.vue";
import router from "./router";
import { useAuthStore } from "./stores/auth";
import { i18n, applyTranslationOverrides } from "./i18n";
import { translationsApi } from "./api/translations";
import "./assets/main.css";

const app = createApp(App);

app.use(createPinia());
app.use(VueQueryPlugin);
app.use(PrimeVue, { unstyled: true });
app.use(router);
app.use(i18n);

const authStore = useAuthStore();

Promise.all([
  authStore.init(),
  translationsApi.publicOverrides().then(applyTranslationOverrides).catch(() => undefined),
]).finally(() => {
  app.mount("#app");
});
