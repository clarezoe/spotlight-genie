<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  LayoutGrid,
  File,
  FileText,
  FileCode,
  FileJson,
  FileSpreadsheet,
  Calculator,
  Globe,
  Moon,
  Lock,
  Music,
  DollarSign,
  Clipboard,
  Settings,
  Image,
  Video,
  Archive,
  Presentation,
  Type,
  Database,
  Folder,
  Users,
} from "lucide-vue-next";
import type { SearchResult } from "../types";
import CategoryBadge from "./CategoryBadge.vue";

const props = defineProps<{
  result: SearchResult;
  selected: boolean;
}>();

defineEmits<{
  action: [result: SearchResult];
}>();

const iconMap: Record<string, typeof LayoutGrid> = {
  "layout-grid": LayoutGrid,
  file: File,
  "file-text": FileText,
  "file-code": FileCode,
  "file-json": FileJson,
  "file-spreadsheet": FileSpreadsheet,
  calculator: Calculator,
  globe: Globe,
  moon: Moon,
  lock: Lock,
  music: Music,
  "dollar-sign": DollarSign,
  clipboard: Clipboard,
  settings: Settings,
  image: Image,
  video: Video,
  archive: Archive,
  presentation: Presentation,
  type: Type,
  database: Database,
  folder: Folder,
  users: Users,
};

const lazyIcon = ref<string | null>(null);
const iconRequestId = ref(0);

const effectiveIcon = computed(() => {
  if (lazyIcon.value) return lazyIcon.value;
  return props.result.icon;
});

const isImageIcon = computed(() => effectiveIcon.value?.startsWith("data:"));
const IconComponent = computed(() => iconMap[effectiveIcon.value] || Globe);

const iconColor = computed(() => {
  const colors: Record<string, string> = {
    SPOTIFY: "#1DB954",
    CURRENCY: "#3B82F6",
    CLIP: "#8B5CF6",
  };
  return colors[props.result.category] || "#94A3B8";
});

async function loadAppIcon() {
  lazyIcon.value = null;
  if (props.result.category !== "APP" || props.result.icon?.startsWith("data:")) return;
  const requestId = ++iconRequestId.value;
  try {
    const icon = await invoke<string | null>("get_app_icon", { appPath: props.result.action_data });
    if (requestId === iconRequestId.value && icon) {
      lazyIcon.value = icon;
    }
  } catch {
    // Ignore errors, keep default icon
  }
}

watch(
  () => [props.result.id, props.result.action_data, props.result.icon],
  () => {
    void loadAppIcon();
  },
  { immediate: true }
);
</script>

<template>
  <button
    class="flex w-full cursor-pointer items-center gap-3 px-5 py-2 transition-colors duration-150"
    :class="{
      'border-l-3 border-l-genie-accent bg-genie-selected': selected,
      'border-l-3 border-l-transparent hover:bg-genie-hover': !selected,
    }"
    @click="$emit('action', result)"
  >
    <div
      class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg"
      style="background: rgba(255, 255, 255, 0.08)"
    >
      <img v-if="isImageIcon" :src="effectiveIcon" class="h-6 w-6 rounded" alt="" />
      <component v-else :is="IconComponent" :size="18" :color="iconColor" />
    </div>
    <div class="flex min-w-0 flex-1 flex-col gap-0.5">
      <span class="truncate text-left font-body text-sm font-medium text-genie-text">
        {{ result.title }}
      </span>
      <span class="truncate text-left font-body text-xs text-genie-text-secondary">
        {{ result.subtitle }}
      </span>
    </div>
    <CategoryBadge :category="result.category" />
  </button>
</template>
