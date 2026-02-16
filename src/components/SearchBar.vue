<script setup lang="ts">
import { ref, nextTick, onMounted } from "vue";
import { Search } from "lucide-vue-next";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { PluginKeywordMatch } from "../types";
import { KEYWORD_BADGE_COLORS } from "../types";

const props = defineProps<{
  modelValue: string;
  activeKeyword: PluginKeywordMatch | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
  keydown: [event: KeyboardEvent];
}>();

const inputRef = ref<HTMLInputElement | null>(null);

function focusInput() {
  nextTick(() => inputRef.value?.focus());
}

onMounted(() => {
  focusInput();
  listen("genie:focus", () => focusInput()).catch(() => {});
});

const isMac = navigator.platform.toUpperCase().includes("MAC");
const shortcutLabel = isMac ? "⌘K" : "Ctrl+K";

async function onDragStart(e: MouseEvent) {
  const tag = (e.target as HTMLElement)?.tagName;
  if (tag === "INPUT") return;
  e.preventDefault();
  try {
    await getCurrentWindow().startDragging();
  } catch {
    // NOTE: no-op outside Tauri
  }
}
</script>

<template>
  <div
    data-tauri-drag-region
    class="flex w-full items-center gap-3 px-5 py-4"
    style="border-bottom: 1px solid rgba(255, 255, 255, 0.06); cursor: grab;"
    @mousedown.left="onDragStart"
  >
    <Search :size="20" class="shrink-0 text-genie-text-muted" />
    <span
      v-if="activeKeyword"
      class="shrink-0 rounded-full px-2.5 py-1 font-heading text-xs font-semibold"
      :style="{
        background: KEYWORD_BADGE_COLORS[activeKeyword.plugin.keyword ?? ''] || '#4B5563',
        color: '#0B1020',
      }"
    >
      [{{ activeKeyword.plugin.keyword }}]
    </span>
    <input
      ref="inputRef"
      type="text"
      :value="props.modelValue"
      placeholder="Search apps, files, commands..."
      class="min-w-0 flex-1 border-0 bg-transparent font-heading text-xl text-genie-text outline-none placeholder:text-genie-text-muted"
      role="combobox"
      aria-expanded="true"
      aria-autocomplete="list"
      aria-controls="results-list"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      @keydown="emit('keydown', $event)"
    />
    <span
      class="shrink-0 rounded-lg border px-2 py-1 font-body text-[11px] text-genie-text-muted"
      style="background: rgba(255, 255, 255, 0.04); border-color: rgba(255, 255, 255, 0.08)"
    >
      {{ shortcutLabel }}
    </span>
  </div>
</template>
