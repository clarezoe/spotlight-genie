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
let latestRequestId = 0;

export function useSearch() {
  const { matchKeyword, searchPlugins } = usePlugins();

  watch(query, (val) => {
    if (debounceTimer) clearTimeout(debounceTimer);
    const match = matchKeyword(val);
    activeKeyword.value = match;
    const requestId = ++latestRequestId;

    debounceTimer = setTimeout(async () => {
      await performSearch(val, match, requestId);
    }, 50);
  });

  async function performSearch(
    q: string,
    kwMatch: PluginKeywordMatch | null,
    requestId: number
  ) {
    if (!q.trim()) {
      if (requestId !== latestRequestId) return;
      results.value = [];
      selectedIndex.value = 0;
      return;
    }

    if (requestId === latestRequestId) {
      isLoading.value = true;
    }

    let nextResults: SearchResult[] = [];
    try {
      if (kwMatch) {
        nextResults = await kwMatch.plugin.onSearch(kwMatch.query);
      } else {
        const withTimeout = <T>(p: Promise<T>, ms: number, fallback: T) =>
          Promise.race([p, new Promise<T>((r) => setTimeout(() => r(fallback), ms))]);
        const [pluginResults, rustResults] = await Promise.all([
          withTimeout(searchPlugins(q), 400, [] as SearchResult[]),
          invoke<SearchResult[]>("search", { query: q }).catch(() => [] as SearchResult[]),
        ]);
        const merged = [...pluginResults, ...rustResults];
        const normalized = q.trim().toLowerCase();
        merged.sort((a, b) => {
          const aRank = a.score + queryMatchBoost(a, normalized);
          const bRank = b.score + queryMatchBoost(b, normalized);
          if (aRank !== bRank) return bRank - aRank;
          return a.title.length - b.title.length;
        });
        nextResults = merged.slice(0, 8);
      }
      if (requestId !== latestRequestId) return;
      results.value = nextResults;
      selectedIndex.value = 0;
    } finally {
      if (requestId === latestRequestId) {
        isLoading.value = false;
      }
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

function queryMatchBoost(result: SearchResult, normalizedQuery: string): number {
  if (!normalizedQuery) return 0;
  const title = result.title.toLowerCase();
  if (title === normalizedQuery) return 1800;
  if (title.startsWith(normalizedQuery)) return 900;
  const idx = title.indexOf(normalizedQuery);
  if (idx >= 0) return 500 - Math.min(idx * 20, 320);
  return 0;
}
