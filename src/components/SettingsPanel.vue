<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { X, Settings, Keyboard, Monitor, Zap, RotateCcw, FolderPlus, Trash2, Plug } from "lucide-vue-next";
import { useTheme } from "../composables/useTheme";
import { usePlugins } from "../composables/usePlugins";

interface AppSettings {
  hotkey: string;
  max_results: number;
  launch_at_login: boolean;
  theme: string;
  show_recent_apps: boolean;
  search_folders: string[];
  disabled_plugins?: string[];
}

const { applyTheme } = useTheme();
const { plugins } = usePlugins();
const emit = defineEmits<{ close: [] }>();
const settings = ref<AppSettings>({
  hotkey: "CommandOrControl+Space",
  max_results: 8,
  launch_at_login: false,
  theme: "dark",
  show_recent_apps: true,
  search_folders: [],
  disabled_plugins: [],
});
const saving = ref(false);
const saved = ref(false);
const capturingHotkey = ref(false);
const capturedKeys = ref(new Set<string>());

onMounted(async () => {
  try {
    settings.value = await invoke<AppSettings>("get_settings");
    if (!settings.value.disabled_plugins) {
      settings.value.disabled_plugins = [];
    }
  } catch {
    // NOTE: use defaults
  }
});

function isPluginEnabled(pluginId: string): boolean {
  return !settings.value.disabled_plugins?.includes(pluginId);
}

function togglePlugin(pluginId: string) {
  if (!settings.value.disabled_plugins) {
    settings.value.disabled_plugins = [];
  }
  const index = settings.value.disabled_plugins.indexOf(pluginId);
  if (index > -1) {
    settings.value.disabled_plugins.splice(index, 1);
  } else {
    settings.value.disabled_plugins.push(pluginId);
  }
}

async function startCapture() {
  capturingHotkey.value = true;
  capturedKeys.value.clear();
  await invoke("set_capturing_shortcut", { capturing: true }).catch(() => {});
  await invoke("unregister_global_shortcut").catch(() => {});
}

async function stopCapture() {
  if (!capturingHotkey.value) return;
  capturingHotkey.value = false;
  capturedKeys.value.clear();
  await invoke("set_capturing_shortcut", { capturing: false }).catch(() => {});
  await invoke("register_global_shortcut").catch(() => {});
}

function onKeyDown(e: KeyboardEvent) {
  if (!capturingHotkey.value) return;

  e.preventDefault();
  e.stopPropagation();

  const keys: string[] = [];

  if (e.metaKey) keys.push("Command");
  if (e.ctrlKey) keys.push("Control");
  if (e.altKey) keys.push("Alt");
  if (e.shiftKey) keys.push("Shift");

  const key = e.key;
  if (key && !["Control", "Alt", "Shift", "Meta", "Command"].includes(key)) {
    // NOTE: handle special keys like Space, Enter, Tab, etc.
    const specialKeyMap: Record<string, string> = {
      " ": "Space",
      Enter: "Enter",
      Tab: "Tab",
      Escape: "Escape",
      Backspace: "Backspace",
      Delete: "Delete",
      ArrowUp: "Up",
      ArrowDown: "Down",
      ArrowLeft: "Left",
      ArrowRight: "Right",
      Home: "Home",
      End: "End",
      PageUp: "PageUp",
      PageDown: "PageDown",
    };
    const mappedKey = specialKeyMap[key] || (key.length === 1 ? key.toUpperCase() : key);
    keys.push(mappedKey);
  }

  if (keys.length >= 2 || (keys.length === 1 && !keys[0].match(/^(Control|Alt|Shift|Command)$/))) {
    settings.value.hotkey = keys.join("+").replace("Command", "CommandOrControl");
    stopCapture();
  }
}

function formatHotkeyDisplay(hotkey: string): string {
  return hotkey
    .replace("CommandOrControl", "⌘/Ctrl")
    .replace("Command", "⌘")
    .replace("Control", "Ctrl")
    .replace("Alt", "⌥")
    .replace("Shift", "⇧")
    .replace("Space", "Space")
    .replace("Plus", "+");
}

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
    disabled_plugins: [],
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
        <div
        class="w-full rounded-lg border px-3 py-2 font-body text-xs text-genie-text outline-none focus-within:border-genie-accent cursor-pointer select-none transition-colors"
        :class="capturingHotkey ? 'border-genie-accent bg-genie-accent/10' : 'border-white/10 bg-white/5'"
        tabindex="0"
        @click="startCapture"
        @blur="stopCapture"
        @keydown="onKeyDown"
      >
        <span v-if="capturingHotkey" class="text-genie-accent animate-pulse">Press keys...</span>
        <span v-else>{{ formatHotkeyDisplay(settings.hotkey) }}</span>
      </div>
      <p class="text-[10px] text-genie-text-muted">Click and press your desired shortcut</p>
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

      <div class="space-y-2">
        <label class="flex items-center gap-2 text-xs font-medium text-genie-text-secondary">
          <Plug :size="12" /> Plugins
        </label>
        <div class="space-y-1">
          <label
            v-for="plugin in plugins"
            :key="plugin.id"
            class="flex cursor-pointer items-center justify-between rounded-lg border border-white/10 bg-white/5 px-3 py-2 hover:bg-white/10"
          >
            <div class="flex items-center gap-2">
              <span class="text-xs text-genie-text">{{ plugin.name }}</span>
            </div>
            <input
              type="checkbox"
              :checked="isPluginEnabled(plugin.id)"
              @change="togglePlugin(plugin.id)"
              class="accent-amber-500"
            />
          </label>
        </div>
        <p class="text-[10px] text-genie-text-muted">Enable or disable plugins. Changes take effect after saving.</p>
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
