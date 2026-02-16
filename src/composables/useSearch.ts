import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SearchResult, PluginKeywordMatch } from "../types";
import { usePlugins } from "./usePlugins";

const query = ref("");
const results = ref<SearchResult[]>([]);
const selectedIndex = ref(0);
const isLoading = ref(false);
const activeKeyword = ref<PluginKeywordMatch | null>(null);

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

export function useSearch() {
  const { matchKeyword, searchPlugins } = usePlugins();

  watch(query, (val) => {
    if (debounceTimer) clearTimeout(debounceTimer);
    const match = matchKeyword(val);
    activeKeyword.value = match;

    debounceTimer = setTimeout(async () => {
      await performSearch(val, match);
    }, 50);
  });

  async function performSearch(q: string, kwMatch: PluginKeywordMatch | null) {
    if (!q.trim()) {
      results.value = [];
      selectedIndex.value = 0;
      return;
    }

    isLoading.value = true;
    try {
      if (kwMatch) {
        results.value = await kwMatch.plugin.onSearch(kwMatch.query);
      } else {
        const pluginResults = await searchPlugins(q);
        let rustResults: SearchResult[] = [];
        try {
          rustResults = await invoke<SearchResult[]>("search", { query: q });
        } catch {
          // NOTE: invoke fails outside Tauri webview
        }
        const merged = [...pluginResults, ...rustResults];
        merged.sort((a, b) => b.score - a.score);
        results.value = merged.slice(0, 8);
      }
      selectedIndex.value = 0;
    } finally {
      isLoading.value = false;
    }
  }

  function clear() {
    query.value = "";
    results.value = [];
    selectedIndex.value = 0;
    activeKeyword.value = null;
  }

  return {
    query,
    results,
    selectedIndex,
    isLoading,
    activeKeyword,
    clear,
  };
}
