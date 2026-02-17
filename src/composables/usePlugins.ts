import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { GeniePlugin, SearchResult, PluginKeywordMatch } from "../types";

const plugins = ref<GeniePlugin[]>([]);
const disabledPlugins = ref<string[]>([]);

export function usePlugins() {
  const enabledPlugins = computed(() =>
    plugins.value.filter((p) => !disabledPlugins.value.includes(p.id))
  );

  async function loadDisabledPlugins() {
    try {
      const settings = await invoke<{ disabled_plugins?: string[] }>("get_settings");
      disabledPlugins.value = settings.disabled_plugins || [];
    } catch {
      disabledPlugins.value = [];
    }
  }

  function register(plugin: GeniePlugin) {
    if (!plugins.value.find((p) => p.id === plugin.id)) {
      plugins.value.push(plugin);
      plugin.onInit?.();
    }
  }

  function unregister(pluginId: string) {
    const idx = plugins.value.findIndex((p) => p.id === pluginId);
    if (idx !== -1) {
      plugins.value[idx].onDestroy?.();
      plugins.value.splice(idx, 1);
    }
  }

  function matchKeyword(query: string): PluginKeywordMatch | null {
    const trimmed = query.trim();
    for (const plugin of enabledPlugins.value) {
      if (plugin.keyword && trimmed.startsWith(plugin.keyword + " ")) {
        return {
          plugin,
          query: trimmed.slice(plugin.keyword.length + 1),
        };
      }
    }
    return null;
  }

  async function searchPlugins(query: string): Promise<SearchResult[]> {
    const globalPlugins = enabledPlugins.value.filter((p) => !p.keyword);
    const allResults: SearchResult[] = [];
    const promises = globalPlugins.map((p) =>
      p.onSearch(query).catch(() => [] as SearchResult[])
    );
    const settled = await Promise.all(promises);
    for (const batch of settled) {
      allResults.push(...batch);
    }
    return allResults;
  }

  return {
    plugins,
    enabledPlugins,
    register,
    unregister,
    matchKeyword,
    searchPlugins,
    loadDisabledPlugins,
  };
}
