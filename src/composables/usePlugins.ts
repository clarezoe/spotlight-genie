import { ref } from "vue";
import type { GeniePlugin, SearchResult, PluginKeywordMatch } from "../types";

const plugins = ref<GeniePlugin[]>([]);

export function usePlugins() {
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
    for (const plugin of plugins.value) {
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
    const globalPlugins = plugins.value.filter((p) => !p.keyword);
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
    register,
    unregister,
    matchKeyword,
    searchPlugins,
  };
}
