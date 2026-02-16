import { invoke } from "@tauri-apps/api/core";
import type { Ref } from "vue";
import type { SearchResult } from "../types";

interface UseKeyboardOptions {
  results: Ref<SearchResult[]>;
  selectedIndex: Ref<number>;
  query: Ref<string>;
  clear: () => void;
  onAction: (result: SearchResult) => void;
}

export function useKeyboard(opts: UseKeyboardOptions) {
  function handleKeydown(e: KeyboardEvent) {
    const { results, selectedIndex, query, clear, onAction } = opts;

    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        if (results.value.length > 0) {
          selectedIndex.value = (selectedIndex.value + 1) % results.value.length;
        }
        break;

      case "ArrowUp":
        e.preventDefault();
        if (results.value.length > 0) {
          selectedIndex.value =
            selectedIndex.value <= 0
              ? results.value.length - 1
              : selectedIndex.value - 1;
        }
        break;

      case "Enter":
        e.preventDefault();
        if (results.value.length > 0 && results.value[selectedIndex.value]) {
          onAction(results.value[selectedIndex.value]);
        }
        break;

      case "Escape":
        e.preventDefault();
        if (query.value) {
          clear();
        } else {
          invoke("hide_window");
        }
        break;
    }

    if ((e.metaKey || e.ctrlKey) && e.key >= "1" && e.key <= "8") {
      e.preventDefault();
      const idx = parseInt(e.key) - 1;
      if (idx < results.value.length) {
        selectedIndex.value = idx;
        onAction(results.value[idx]);
      }
    }
  }

  return { handleKeydown };
}
