<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useEditor, EditorContent } from "@tiptap/vue-3";
import StarterKit from "@tiptap/starter-kit";
import AppShell from "@/components/AppShell.vue";
import { useAuthStore } from "@/stores/auth";
import {
  useNotesQuery,
  useCreateNoteMutation,
  useToggleNoteMutation,
  useDeleteNoteMutation,
} from "@/composables/useNotesQueries";

const { t } = useI18n();
const authStore = useAuthStore();

const { data: notes, isLoading } = useNotesQuery();
const createMutation = useCreateNoteMutation();
const toggleMutation = useToggleNoteMutation();
const deleteMutation = useDeleteNoteMutation();

// ── Editor ───────────────────────────────────────────────
const editor = useEditor({
  extensions: [StarterKit],
  editorProps: {
    attributes: {
      class: "focus:outline-none min-h-[80px] px-4 py-3 text-sm text-gray-800 leading-relaxed",
    },
  },
});

const editorEmpty = computed(() => {
  if (!editor.value) return true;
  return editor.value.getText().trim().length === 0;
});

async function handleAdd() {
  if (!editor.value || editorEmpty.value) return;
  await createMutation.mutateAsync(editor.value.getHTML());
  editor.value.commands.clearContent();
}

// ── Corner color per note (derived from id) ──────────────
const CORNER_COLORS = ["#fde047", "#86efac", "#93c5fd", "#d8b4fe", "#fdba74", "#fda4af"];

function cornerColor(id: string) {
  const n = id.split("").reduce((a, c) => a + c.charCodeAt(0), 0);
  return CORNER_COLORS[n % CORNER_COLORS.length];
}

// ── Relative date ────────────────────────────────────────
function relativeDate(iso: string) {
  const diff = (Date.now() - new Date(iso).getTime()) / 1000;
  if (diff < 60) return t("dashboard.dateJustNow");
  if (diff < 3600) return t("dashboard.dateMinutesAgo", { n: Math.floor(diff / 60) });
  if (diff < 86400) return t("dashboard.dateHoursAgo", { n: Math.floor(diff / 3600) });
  if (diff < 172800) return t("dashboard.dateYesterday");
  return new Date(iso).toLocaleDateString();
}
</script>

<template>
  <AppShell>
    <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">

      <!-- ── Editor panel ─────────────────────────────── -->
      <div class="lg:col-span-1">
        <div class="bg-white rounded-xl border border-gray-200 shadow-sm overflow-hidden sticky top-6">
          <div class="px-4 pt-4 pb-2 border-b border-gray-100">
            <p class="text-xs font-semibold text-gray-500 uppercase tracking-wide">
              {{ t("dashboard.newNote") }}
            </p>
          </div>

          <!-- Toolbar -->
          <div class="flex items-center gap-1 px-3 py-2 border-b border-gray-100 bg-gray-50">
            <button
              :class="['w-7 h-7 rounded flex items-center justify-center text-xs font-bold transition-colors',
                editor?.isActive('bold') ? 'bg-gray-800 text-white' : 'text-gray-500 hover:bg-gray-200']"
              title="Bold"
              @mousedown.prevent="editor?.chain().focus().toggleBold().run()"
            >B</button>
            <button
              :class="['w-7 h-7 rounded flex items-center justify-center text-xs italic font-semibold transition-colors',
                editor?.isActive('italic') ? 'bg-gray-800 text-white' : 'text-gray-500 hover:bg-gray-200']"
              title="Italic"
              @mousedown.prevent="editor?.chain().focus().toggleItalic().run()"
            >I</button>
            <div class="w-px h-4 bg-gray-200 mx-1" />
            <button
              :class="['w-7 h-7 rounded flex items-center justify-center transition-colors',
                editor?.isActive('bulletList') ? 'bg-gray-800 text-white' : 'text-gray-500 hover:bg-gray-200']"
              title="Bullet list"
              @mousedown.prevent="editor?.chain().focus().toggleBulletList().run()"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="9" y1="6" x2="20" y2="6"/><line x1="9" y1="12" x2="20" y2="12"/><line x1="9" y1="18" x2="20" y2="18"/>
                <circle cx="4" cy="6" r="1.5" fill="currentColor" stroke="none"/>
                <circle cx="4" cy="12" r="1.5" fill="currentColor" stroke="none"/>
                <circle cx="4" cy="18" r="1.5" fill="currentColor" stroke="none"/>
              </svg>
            </button>
          </div>

          <EditorContent :editor="editor" />

          <div class="px-3 pb-3 pt-1">
            <button
              :disabled="editorEmpty || createMutation.isPending.value"
              class="w-full rounded-lg bg-gray-900 text-white py-2 text-sm font-medium hover:bg-gray-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
              @click="handleAdd"
            >
              {{ t("dashboard.noteAdd") }}
            </button>
          </div>
        </div>
      </div>

      <!-- ── Notes grid ───────────────────────────────── -->
      <div class="lg:col-span-3">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-sm font-semibold text-gray-700">{{ t("dashboard.notes") }}</h2>
          <span class="text-xs text-gray-400">
            {{ notes?.filter(n => !n.done).length ?? 0 }} {{ t("dashboard.notesRemaining") }}
          </span>
        </div>

        <div v-if="isLoading" class="py-16 text-sm text-gray-400 text-center">…</div>

        <p v-else-if="!notes?.length" class="py-16 text-sm text-gray-400 text-center">
          {{ t("dashboard.notesEmpty") }}
        </p>

        <div v-else class="grid grid-cols-2 xl:grid-cols-3 gap-4">
          <div
            v-for="note in notes"
            :key="note.id"
            class="group relative bg-white rounded-xl border border-gray-100 flex flex-col shadow-md hover:shadow-xl transition-shadow duration-200"
            :class="note.done ? 'opacity-60' : ''"
          >
            <!-- Left color strip -->
            <div
              class="absolute left-0 top-0 bottom-0 w-1 rounded-l-xl"
              :style="{ background: note.done ? '#e5e7eb' : cornerColor(note.id) }"
            />

            <!-- Body -->
            <div class="flex-1 pl-5 pr-4 pt-4 pb-2">
              <div
                class="text-sm note-body leading-relaxed"
                :class="note.done ? 'text-gray-400 line-through' : 'text-gray-800'"
                v-html="note.body"
              />
            </div>

            <!-- Footer -->
            <div class="flex items-center justify-between pl-5 pr-4 pb-3 mt-1">
              <div class="min-w-0 mr-2">
                <p class="text-xs font-medium text-gray-500 truncate">{{ note.created_by_name }}</p>
                <p class="text-xs text-gray-400">{{ relativeDate(note.created_at) }}</p>
              </div>

              <div class="flex items-center gap-1.5 shrink-0">
                <!-- Toggle done -->
                <button
                  class="w-5 h-5 rounded-full border-2 flex items-center justify-center transition-all"
                  :class="note.done
                    ? 'bg-gray-400 border-gray-400'
                    : 'border-gray-300 hover:border-gray-500'"
                  :disabled="toggleMutation.isPending.value"
                  @click="toggleMutation.mutate({ id: note.id, done: !note.done })"
                >
                  <svg v-if="note.done" xmlns="http://www.w3.org/2000/svg" width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="20 6 9 17 4 12"/>
                  </svg>
                </button>

                <!-- Delete -->
                <button
                  class="opacity-0 group-hover:opacity-100 w-5 h-5 rounded flex items-center justify-center text-gray-300 hover:text-red-500 hover:bg-red-50 transition-all"
                  :disabled="deleteMutation.isPending.value"
                  @click="deleteMutation.mutate(note.id)"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="3 6 5 6 21 6"/>
                    <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
                    <path d="M10 11v6M14 11v6"/>
                    <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/>
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </AppShell>
</template>

<style scoped>
.note-body :deep(p) { margin: 0 0 0.25rem; }
.note-body :deep(p:last-child) { margin-bottom: 0; }
.note-body :deep(ul) { list-style: disc; padding-left: 1.1rem; margin: 0.25rem 0; }
.note-body :deep(ol) { list-style: decimal; padding-left: 1.1rem; margin: 0.25rem 0; }
.note-body :deep(li) { margin: 0.1rem 0; }
.note-body :deep(strong) { font-weight: 600; }
.note-body :deep(em) { font-style: italic; }
</style>
