<script setup lang="ts">
import { nextTick, reactive, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import AppShell from "@/components/AppShell.vue";
import { ApiError } from "@/api/client";
import type { Room } from "@/api/rooms";
import {
  useRoomsQuery,
  useCreateRoomMutation,
  useUpdateRoomMutation,
  useToggleRoomMutation,
  useDeleteRoomMutation,
} from "@/composables/useRoomsQueries";
import { useSettingsStore } from "@/stores/settings";

const { t } = useI18n();
const settingsStore = useSettingsStore();

const { data: rooms, isLoading } = useRoomsQuery();
const createRoomMutation = useCreateRoomMutation();
const updateRoomMutation = useUpdateRoomMutation();
const toggleRoomMutation = useToggleRoomMutation();
const deleteRoomMutation = useDeleteRoomMutation();

const dialogVisible = ref(false);
const dialogEl = ref<HTMLDialogElement>();
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

watch(dialogVisible, (v) => {
  if (v) nextTick(() => dialogEl.value?.showModal());
  else dialogEl.value?.close();
});

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
      await updateRoomMutation.mutateAsync({ id: editingRoom.value.id, input: { ...form } });
      successMessage.value = t("adminRooms.saved");
    } else {
      await createRoomMutation.mutateAsync({ ...form });
      successMessage.value = t("adminRooms.added");
    }
    dialogVisible.value = false;
  } catch (err) {
    errorMessage.value = err instanceof ApiError ? err.message : t("adminRooms.genericError");
  }
}

async function handleToggle(room: Room) {
  errorMessage.value = "";
  successMessage.value = "";
  pendingRoomId.value = room.id;
  try {
    await toggleRoomMutation.mutateAsync(room.id);
  } catch (err) {
    errorMessage.value = err instanceof ApiError ? err.message : t("adminRooms.genericError");
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
    errorMessage.value = err instanceof ApiError ? err.message : t("adminRooms.genericError");
  } finally {
    pendingRoomId.value = null;
  }
}

function formatDate(iso: string) {
  return settingsStore.formatDate(iso);
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
        <button class="btn btn-sm bg-gray-900 text-white hover:bg-gray-700 border-none" @click="openAddDialog">
          {{ t("adminRooms.add") }}
        </button>
      </div>

      <div v-if="errorMessage" class="alert alert-error mt-3 text-sm">{{ errorMessage }}</div>
      <div v-if="successMessage" class="alert alert-success mt-3 text-sm">{{ successMessage }}</div>

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
            <td colspan="6" class="py-8 text-center text-sm text-gray-400">{{ t("adminRooms.emptyState") }}</td>
          </tr>
          <tr v-for="room in rooms" :key="room.id" class="border-b border-gray-100">
            <td class="py-2 pr-4 text-gray-900">{{ room.room_number }}</td>
            <td class="py-2 pr-4 text-gray-900">{{ room.room_name }}</td>
            <td class="py-2 pr-4 text-gray-600">{{ formatPrice(room.price) }}</td>
            <td class="py-2 pr-4">
              <span :class="['badge badge-sm', room.is_active ? 'badge-success' : 'badge-ghost']">
                {{ room.is_active ? t("adminRooms.active") : t("adminRooms.inactive") }}
              </span>
            </td>
            <td class="py-2 pr-4 text-gray-600">{{ formatDate(room.updated_at) }}</td>
            <td class="py-2 flex items-center gap-2">
              <button class="btn btn-ghost btn-xs text-blue-600" @click="openEditDialog(room)">
                {{ t("adminRooms.edit") }}
              </button>
              <button
                class="btn btn-ghost btn-xs text-yellow-600 disabled:opacity-50"
                :disabled="pendingRoomId === room.id"
                @click="handleToggle(room)"
              >
                {{ room.is_active ? t("adminRooms.disable") : t("adminRooms.enable") }}
              </button>
              <button
                class="btn btn-ghost btn-xs text-red-600 disabled:opacity-50"
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

    <dialog ref="dialogEl" class="modal" @close="dialogVisible = false">
      <div class="modal-box w-full max-w-lg">
        <div class="flex items-center justify-between mb-4">
          <h3 class="font-semibold text-gray-900">
            {{ editingRoom ? t("adminRooms.editTitle") : t("adminRooms.addTitle") }}
          </h3>
          <form method="dialog">
            <button class="btn btn-sm btn-circle btn-ghost">✕</button>
          </form>
        </div>

        <form id="room-form" class="flex flex-col gap-5" @submit.prevent="handleSubmit">
          <div class="grid grid-cols-2 gap-4">
            <div class="flex flex-col gap-1.5">
              <label class="text-xs font-medium text-gray-700">
                {{ t("adminRooms.roomNumber") }} <span class="text-red-500 ml-0.5">*</span>
              </label>
              <input
                v-model="form.room_number"
                required
                class="input input-bordered input-sm w-full"
                :placeholder="t('adminRooms.roomNumberPlaceholder')"
              />
            </div>
            <div class="flex flex-col gap-1.5">
              <label class="text-xs font-medium text-gray-700">
                {{ t("adminRooms.roomName") }} <span class="text-red-500 ml-0.5">*</span>
              </label>
              <input
                v-model="form.room_name"
                required
                class="input input-bordered input-sm w-full"
                :placeholder="t('adminRooms.roomNamePlaceholder')"
              />
            </div>
          </div>

          <div class="flex flex-col gap-1.5">
            <label class="text-xs font-medium text-gray-700">
              {{ t("adminRooms.roomDescription") }}
              <span class="ml-1 text-xs font-normal text-gray-400">({{ t("adminRooms.optional") }})</span>
            </label>
            <textarea
              v-model="form.room_description"
              class="textarea textarea-bordered textarea-sm w-full resize-none"
              rows="3"
              :placeholder="t('adminRooms.roomDescriptionPlaceholder')"
            />
          </div>

          <div class="grid grid-cols-2 gap-4 items-end">
            <div class="flex flex-col gap-1.5">
              <label class="text-xs font-medium text-gray-700">
                {{ t("adminRooms.price") }} <span class="text-red-500 ml-0.5">*</span>
              </label>
              <div class="relative">
                <span class="absolute inset-y-0 left-3 flex items-center text-sm text-gray-400 pointer-events-none">₭</span>
                <input
                  v-model.number="form.price"
                  type="number"
                  min="0"
                  step="1000"
                  required
                  class="input input-bordered input-sm w-full pl-7"
                />
              </div>
            </div>

            <div class="flex flex-col gap-1.5 pb-0.5">
              <label class="text-xs font-medium text-gray-700">{{ t("adminRooms.status") }}</label>
              <div class="flex items-center gap-3">
                <input v-model="form.is_active" type="checkbox" class="toggle toggle-sm" />
                <span class="text-sm" :class="form.is_active ? 'text-gray-800' : 'text-gray-400'">
                  {{ form.is_active ? t("adminRooms.active") : t("adminRooms.inactive") }}
                </span>
              </div>
            </div>
          </div>

          <div v-if="errorMessage" class="alert alert-error text-sm">{{ errorMessage }}</div>
        </form>

        <div class="flex justify-end gap-3 mt-6 pt-4 border-t border-gray-100">
          <form method="dialog">
            <button class="btn btn-sm btn-outline">{{ t("adminRooms.cancel") }}</button>
          </form>
          <button
            type="submit"
            form="room-form"
            class="btn btn-sm bg-gray-900 text-white hover:bg-gray-700 border-none"
            :class="isPending() ? 'loading' : ''"
            :disabled="isPending()"
          >
            {{ t("adminRooms.save") }}
          </button>
        </div>
      </div>
      <form method="dialog" class="modal-backdrop"><button>close</button></form>
    </dialog>
  </AppShell>
</template>
