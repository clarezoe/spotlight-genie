<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import SearchBar from "./components/SearchBar.vue";
import ResultList from "./components/ResultList.vue";
import ActionBar from "./components/ActionBar.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import { useSearch } from "./composables/useSearch";
import { useKeyboard } from "./composables/useKeyboard";
import { usePlugins } from "./composables/usePlugins";
import { useWindowSize } from "./composables/useWindowSize";
import { useTheme } from "./composables/useTheme";
import type { SearchResult } from "./types";
import {
  spotifyPlugin,
  currencyPlugin,
  clipboardPlugin,
  contactsPlugin,
} from "../plugins";

const { register, plugins, loadDisabledPlugins } = usePlugins();
const { query, results, selectedIndex, activeKeyword, clear } = useSearch();
const showSettings = ref(false);
useWindowSize(results);

const { currentTheme, init: initTheme } = useTheme();

const panelStyle = computed(() =>
  currentTheme.value === "light"
    ? {
        background: "rgba(248, 250, 252, 0.95)",
        border: "1px solid rgba(0, 0, 0, 0.08)",
        boxShadow: "0 25px 60px rgba(0, 0, 0, 0.12), inset 0 1px 0 rgba(255, 255, 255, 0.6)",
      }
    : {
        background: "rgba(15, 15, 35, 0.92)",
        border: "1px solid rgba(255, 255, 255, 0.12)",
        boxShadow: "0 25px 60px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.05)",
      }
);

// Watch settings panel and adjust window size
watch(showSettings, async (isShowing) => {
  try {
    const win = getCurrentWindow();
    const pos = await win.outerPosition();
    if (isShowing) {
      // Settings panel needs more height
      await win.setSize(new LogicalSize(696, 600));
    } else {
      // Back to search mode - let useWindowSize handle it
      await win.setSize(new LogicalSize(696, 88));
    }
    await win.setPosition(pos);
  } catch (e) {
    console.warn("Failed to resize window:", e);
  }
});

onMounted(async () => {
  initTheme();
  register(spotifyPlugin);
  register(currencyPlugin);
  register(clipboardPlugin);
  register(contactsPlugin);
  await loadDisabledPlugins();

  // Listen for settings event from tray menu
  await listen("genie:show-settings", () => {
    showSettings.value = true;
  });
});

const selectedResult = computed<SearchResult | null>(
  () => results.value[selectedIndex.value] ?? null
);

const pluginCategories = new Set(["CONTACT", "CLIP", "SPOTIFY", "CURRENCY"]);

async function handleAction(result: SearchResult) {
  if (result.id === "sys:settings") {
    showSettings.value = true;
    return;
  }
  if (result.category === "SYS") {
    await invoke("run_system_command", { command: result.action_data });
  } else if (pluginCategories.has(result.category)) {
    const plugin = plugins.value.find((p) =>
      result.id.startsWith(p.id.split(":")[0]) || p.name.toUpperCase() === result.category
    );
    if (plugin?.onAction) {
      await plugin.onAction(result);
    }
  } else {
    await invoke("launch_item", {
      actionData: result.action_data,
      category: result.category,
    });
  }
  clear();
  await invoke("hide_window");
}

const { handleKeydown } = useKeyboard({
  results,
  selectedIndex,
  query,
  clear,
  onAction: handleAction,
});
</script>

<template>
  <div class="genie-panel flex flex-col overflow-hidden" :style="panelStyle">
    <template v-if="showSettings">
      <SettingsPanel @close="showSettings = false" />
    </template>
    <template v-else>
      <SearchBar
        v-model="query"
        :active-keyword="activeKeyword"
        @keydown="handleKeydown"
      />
      <ResultList
        :results="results"
        :selected-index="selectedIndex"
        @action="handleAction"
      />
      <ActionBar :selected-result="selectedResult" />
    </template>
  </div>
</template>

<style scoped>
.genie-panel {
  width: 100%;
  height: 100%;
  backdrop-filter: blur(24px) saturate(1.2);
  -webkit-backdrop-filter: blur(24px) saturate(1.2);
  border-radius: 16px;
  overflow: hidden;
}
</style>