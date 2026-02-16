<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { X, Settings, Keyboard, Monitor, Zap, RotateCcw, FolderPlus, Trash2 } from "lucide-vue-next";
import { useTheme } from "../composables/useTheme";

interface AppSettings {
  hotkey: string;
  max_results: number;
  launch_at_login: boolean;
  theme: string;
  show_recent_apps: boolean;
  search_folders: string[];
}

const { applyTheme } = useTheme();
const emit = defineEmits<{ close: [] }>();
const settings = ref<AppSettings>({
  hotkey: "CommandOrControl+Space",
  max_results: 8,
  launch_at_login: false,
  theme: "dark",
  show_recent_apps: true,
  search_folders: [],
});
const saving = ref(false);
const saved = ref(false);

onMounted(async () => {
  try {
    settings.value = await invoke<AppSettings>("get_settings");
  } catch {
    // NOTE: use defaults
  }
});

async function addFolder() {
  await invoke("set_suppress_hide", { suppress: true });
  try {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      if (!settings.value.search_folders.includes(selected)) {
        settings.value.search_folders.push(selected);
      }
    }
  } finally {
    await invoke("set_suppress_hide", { suppress: false });
  }
}

function removeFolder(index: number) {
  settings.value.search_folders.splice(index, 1);
}

async function save() {
  saving.value = true;
  try {
    await invoke("save_settings", { settings: settings.value });
    applyTheme(settings.value.theme);
    saved.value = true;
    setTimeout(() => (saved.value = false), 1500);
  } catch {
    // NOTE: save failed silently
  }
  saving.value = false;
}

function reset() {
  settings.value = {
    hotkey: "CommandOrControl+Space",
    max_results: 8,
    launch_at_login: false,
    theme: "dark",
    show_recent_apps: true,
    search_folders: [],
  };
}
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex items-center justify-between px-5 py-3" style="border-bottom: 1px solid rgba(255,255,255,0.06)">
      <div class="flex items-center gap-2">
        <Settings :size="16" color="#CA8A04" />
        <span class="font-display text-sm font-semibold text-genie-text">Settings</span>
      </div>
      <button class="rounded p-1 hover:bg-genie-hover" @click="emit('close')">
        <X :size="14" color="#94A3B8" />
      </button>
    </div>

    <div class="flex-1 space-y-4 overflow-y-auto px-5 py-4">
      <div class="space-y-2">
        <label class="flex items-center gap-2 text-xs font-medium text-genie-text-secondary">
          <Keyboard :size="12" /> Hotkey
        </label>
        <input
          v-model="settings.hotkey"
          class="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 font-body text-xs text-genie-text outline-none focus:border-genie-accent"
        />
      </div>

      <div class="space-y-2">
        <label class="flex items-center gap-2 text-xs font-medium text-genie-text-secondary">
          <Zap :size="12" /> Max Results
        </label>
        <input
          v-model.number="settings.max_results"
          type="number"
          min="3"
          max="20"
          class="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 font-body text-xs text-genie-text outline-none focus:border-genie-accent"
        />
      </div>

      <div class="space-y-2">
        <label class="flex items-center gap-2 text-xs font-medium text-genie-text-secondary">
          <Monitor :size="12" /> Theme
        </label>
        <select
          v-model="settings.theme"
          class="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 font-body text-xs text-genie-text outline-none focus:border-genie-accent"
        >
          <option value="dark">Dark</option>
          <option value="light">Light</option>
          <option value="auto">Auto</option>
        </select>
      </div>

      <label class="flex cursor-pointer items-center justify-between">
        <span class="text-xs font-medium text-genie-text-secondary">Launch at Login</span>
        <input v-model="settings.launch_at_login" type="checkbox" class="accent-amber-500" />
      </label>

      <label class="flex cursor-pointer items-center justify-between">
        <span class="text-xs font-medium text-genie-text-secondary">Show Recent Apps</span>
        <input v-model="settings.show_recent_apps" type="checkbox" class="accent-amber-500" />
      </label>

      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <label class="flex items-center gap-2 text-xs font-medium text-genie-text-secondary">
            <FolderPlus :size="12" /> Search Folders
          </label>
          <button
            class="flex items-center gap-1 rounded px-2 py-0.5 text-[10px] text-genie-accent hover:bg-genie-hover"
            @click="addFolder"
          >
            <FolderPlus :size="10" /> Add
          </button>
        </div>
        <div v-if="settings.search_folders.length" class="space-y-1">
          <div
            v-for="(folder, i) in settings.search_folders"
            :key="folder"
            class="flex items-center justify-between rounded-lg border border-white/10 bg-white/5 px-3 py-1.5"
          >
            <span class="truncate text-[11px] text-genie-text">{{ folder }}</span>
            <button class="ml-2 shrink-0 rounded p-0.5 hover:bg-genie-hover" @click="removeFolder(i)">
              <Trash2 :size="10" color="#94A3B8" />
            </button>
          </div>
        </div>
        <p v-else class="text-[10px] text-genie-text-muted">No folders configured. Click Add to select folders to search.</p>
      </div>
    </div>

    <div class="flex items-center justify-between px-5 py-3" style="border-top: 1px solid rgba(255,255,255,0.06)">
      <button
        class="flex items-center gap-1 rounded px-2 py-1 text-xs text-genie-text-secondary hover:bg-genie-hover"
        @click="reset"
      >
        <RotateCcw :size="11" /> Reset
      </button>
      <button
        class="rounded-lg px-4 py-1.5 text-xs font-medium text-genie-bg transition-colors"
        :class="saved ? 'bg-emerald-500' : 'bg-genie-accent hover:bg-amber-400'"
        :disabled="saving"
        @click="save"
      >
        {{ saved ? "Saved!" : "Save" }}
      </button>
    </div>
  </div>
</template>
