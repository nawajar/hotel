<script setup lang="ts">
import { reactive, ref } from "vue";
import { useI18n } from "vue-i18n";
import AppShell from "@/components/AppShell.vue";
import InputText from "primevue/inputtext";
import InputNumber from "primevue/inputnumber";
import Textarea from "primevue/textarea";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import { ApiError } from "@/api/client";
import type { Room } from "@/api/rooms";
import {
  useRoomsQuery,
  useCreateRoomMutation,
  useUpdateRoomMutation,
  useToggleRoomMutation,
  useDeleteRoomMutation,
} from "@/composables/useRoomsQueries";

const { t } = useI18n();

const { data: rooms, isLoading } = useRoomsQuery();
const createRoomMutation = useCreateRoomMutation();
const updateRoomMutation = useUpdateRoomMutation();
const toggleRoomMutation = useToggleRoomMutation();
const deleteRoomMutation = useDeleteRoomMutation();

const dialogVisible = ref(false);
const editingRoom = ref<Room | null>(null);

const form = reactive({
  room_number: "",
  room_name: "",
  room_description: null as string | null,
  price: 0,
  is_active: true,
});

const errorMessage = ref("");
const successMessage = ref("");

function openAddDialog() {
  editingRoom.value = null;
  form.room_number = "";
  form.room_name = "";
  form.room_description = null;
  form.price = 0;
  form.is_active = true;
  errorMessage.value = "";
  successMessage.value = "";
  dialogVisible.value = true;
}

function openEditDialog(room: Room) {
  editingRoom.value = room;
  form.room_number = room.room_number;
  form.room_name = room.room_name;
  form.room_description = room.room_description;
  form.price = room.price;
  form.is_active = room.is_active;
  errorMessage.value = "";
  successMessage.value = "";
  dialogVisible.value = true;
}

async function handleSubmit() {
  errorMessage.value = "";
  successMessage.value = "";
  try {
    if (editingRoom.value) {
      await updateRoomMutation.mutateAsync({
        id: editingRoom.value.id,
        input: { ...form },
      });
      successMessage.value = t("adminRooms.saved");
    } else {
      await createRoomMutation.mutateAsync({ ...form });
      successMessage.value = t("adminRooms.added");
    }
    dialogVisible.value = false;
  } catch (err) {
    errorMessage.value =
      err instanceof ApiError ? err.message : t("adminRooms.genericError");
  }
}

async function handleToggle(room: Room) {
  errorMessage.value = "";
  successMessage.value = "";
  pendingRoomId.value = room.id;
  try {
    await toggleRoomMutation.mutateAsync(room.id);
  } catch (err) {
    errorMessage.value =
      err instanceof ApiError ? err.message : t("adminRooms.genericError");
  } finally {
    pendingRoomId.value = null;
  }
}

async function handleDelete(room: Room) {
  if (!window.confirm(t("adminRooms.confirmDelete"))) return;
  errorMessage.value = "";
  successMessage.value = "";
  pendingRoomId.value = room.id;
  try {
    await deleteRoomMutation.mutateAsync(room.id);
  } catch (err) {
    errorMessage.value =
      err instanceof ApiError ? err.message : t("adminRooms.genericError");
  } finally {
    pendingRoomId.value = null;
  }
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleString();
}

function formatPrice(price: number) {
  return price.toLocaleString();
}

const isPending = () =>
  createRoomMutation.isPending.value || updateRoomMutation.isPending.value;

const pendingRoomId = ref<string | null>(null);
</script>

<template>
  <AppShell>
    <div class="bg-white rounded-lg border border-gray-200 p-6">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-lg font-semibold text-gray-900">{{ t("adminRooms.title") }}</h1>
          <p class="mt-2 text-sm text-gray-600">{{ t("adminRooms.description") }}</p>
        </div>
        <Button
          :label="t('adminRooms.add')"
          class="rounded-md bg-gray-900 text-white px-4 py-2 text-sm font-medium hover:bg-gray-800"
          @click="openAddDialog"
        />
      </div>

      <p v-if="errorMessage" class="mt-3 text-sm text-red-600">{{ errorMessage }}</p>
      <p v-if="successMessage" class="mt-3 text-sm text-green-600">{{ successMessage }}</p>

      <div v-if="isLoading" class="mt-8 text-sm text-gray-500">…</div>
      <table v-else class="mt-8 w-full text-sm">
        <thead>
          <tr class="text-left text-gray-500 border-b border-gray-200">
            <th class="py-2 pr-4 font-medium">{{ t("adminRooms.roomNumber") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminRooms.roomName") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminRooms.price") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminRooms.status") }}</th>
            <th class="py-2 pr-4 font-medium">{{ t("adminRooms.updatedAt") }}</th>
            <th class="py-2 font-medium"></th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="rooms && rooms.length === 0">
            <td colspan="6" class="py-8 text-center text-sm text-gray-400">
              {{ t("adminRooms.emptyState") }}
            </td>
          </tr>
          <tr v-for="room in rooms" :key="room.id" class="border-b border-gray-100">
            <td class="py-2 pr-4 text-gray-900">{{ room.room_number }}</td>
            <td class="py-2 pr-4 text-gray-900">{{ room.room_name }}</td>
            <td class="py-2 pr-4 text-gray-600">{{ formatPrice(room.price) }}</td>
            <td class="py-2 pr-4 text-gray-600">
              {{ room.is_active ? t("adminRooms.active") : t("adminRooms.inactive") }}
            </td>
            <td class="py-2 pr-4 text-gray-600">{{ formatDate(room.updated_at) }}</td>
            <td class="py-2 flex items-center gap-2">
              <button
                class="text-xs text-blue-600 hover:text-blue-800"
                @click="openEditDialog(room)"
              >
                {{ t("adminRooms.edit") }}
              </button>
              <button
                class="text-xs text-yellow-600 hover:text-yellow-800 disabled:opacity-50 disabled:cursor-not-allowed"
                :disabled="pendingRoomId === room.id"
                @click="handleToggle(room)"
              >
                {{ room.is_active ? t("adminRooms.disable") : t("adminRooms.enable") }}
              </button>
              <button
                class="text-xs text-red-600 hover:text-red-800 disabled:opacity-50 disabled:cursor-not-allowed"
                :disabled="pendingRoomId === room.id"
                @click="handleDelete(room)"
              >
                {{ t("adminRooms.delete") }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <Dialog
      v-model:visible="dialogVisible"
      :header="editingRoom ? t('adminRooms.editTitle') : t('adminRooms.addTitle')"
      modal
      :draggable="false"
      :pt="{
        mask: { class: 'fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4' },
        root: { class: 'bg-white rounded-xl shadow-2xl w-full max-w-[540px] flex flex-col' },
        header: { class: 'flex items-center justify-between px-6 pt-5 pb-0' },
        title: { class: 'text-base font-semibold text-gray-900' },
        closeButton: { class: 'w-8 h-8 flex items-center justify-center rounded-md text-gray-400 hover:text-gray-600 hover:bg-gray-100 transition-colors' },
        closeIcon: { class: 'w-4 h-4' },
        content: { class: 'px-6 py-5' },
        footer: { class: 'px-6 py-4 border-t border-gray-100' },
      }"
    >
      <form id="room-form" class="flex flex-col gap-5 pt-2" @submit.prevent="handleSubmit">
        <!-- Room number + Room name -->
        <div class="grid grid-cols-2 gap-4">
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-gray-700">
              {{ t("adminRooms.roomNumber") }}
              <span class="text-red-500 ml-0.5">*</span>
            </label>
            <InputText
              v-model="form.room_number"
              required
              :placeholder="t('adminRooms.roomNumberPlaceholder')"
              class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400"
            />
          </div>
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-gray-700">
              {{ t("adminRooms.roomName") }}
              <span class="text-red-500 ml-0.5">*</span>
            </label>
            <InputText
              v-model="form.room_name"
              required
              :placeholder="t('adminRooms.roomNamePlaceholder')"
              class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400"
            />
          </div>
        </div>

        <!-- Description -->
        <div class="flex flex-col gap-1.5">
          <label class="text-xs font-medium text-gray-700">
            {{ t("adminRooms.roomDescription") }}
            <span class="ml-1 text-xs font-normal text-gray-400">({{ t("adminRooms.optional") }})</span>
          </label>
          <Textarea
            v-model="form.room_description"
            :placeholder="t('adminRooms.roomDescriptionPlaceholder')"
            rows="3"
            class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm resize-none focus:outline-none focus:ring-2 focus:ring-gray-400"
          />
        </div>

        <!-- Price + Active status -->
        <div class="grid grid-cols-2 gap-4 items-end">
          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-gray-700">
              {{ t("adminRooms.price") }}
              <span class="text-red-500 ml-0.5">*</span>
            </label>
            <div class="relative">
              <span class="absolute inset-y-0 left-3 flex items-center text-sm text-gray-400 pointer-events-none select-none">
                ₭
              </span>
              <InputNumber
                v-model="form.price"
                :min="0"
                :step="1000"
                :use-grouping="true"
                required
                input-class="w-full rounded-md border border-gray-300 pl-7 pr-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-gray-400"
                class="w-full"
              />
            </div>
          </div>

          <div class="flex flex-col gap-1.5 pb-0.5">
            <label class="text-xs font-medium text-gray-700">{{ t("adminRooms.status") }}</label>
            <div class="flex items-center gap-3 h-[38px]">
              <button
                type="button"
                role="switch"
                :aria-checked="form.is_active"
                :class="[
                  'relative inline-flex h-6 w-11 shrink-0 rounded-full transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-gray-400 focus:ring-offset-2',
                  form.is_active ? 'bg-gray-900' : 'bg-gray-200',
                ]"
                @click="form.is_active = !form.is_active"
              >
                <span
                  :class="[
                    'inline-block h-4 w-4 transform rounded-full bg-white shadow transition-transform duration-200 mt-1',
                    form.is_active ? 'translate-x-6' : 'translate-x-1',
                  ]"
                />
              </button>
              <span class="text-sm" :class="form.is_active ? 'text-gray-800' : 'text-gray-400'">
                {{ form.is_active ? t("adminRooms.active") : t("adminRooms.inactive") }}
              </span>
            </div>
          </div>
        </div>

        <p v-if="errorMessage" class="text-sm text-red-600 bg-red-50 border border-red-200 rounded-md px-3 py-2">
          {{ errorMessage }}
        </p>
      </form>

      <template #footer>
        <div class="flex justify-end gap-3 pt-2">
          <Button
            type="button"
            :label="t('adminRooms.cancel')"
            severity="secondary"
            class="rounded-md border border-gray-300 bg-white text-gray-700 px-4 py-2 text-sm font-medium hover:bg-gray-50"
            @click="dialogVisible = false"
          />
          <Button
            type="submit"
            form="room-form"
            :label="t('adminRooms.save')"
            :loading="isPending()"
            class="rounded-md bg-gray-900 text-white px-4 py-2 text-sm font-medium hover:bg-gray-800"
          />
        </div>
      </template>
    </Dialog>
  </AppShell>
</template>
