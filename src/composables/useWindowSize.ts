import { watch, type Ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import type { SearchResult } from "../types";

const SEARCH_BAR_HEIGHT = 64;
const RESULT_ITEM_HEIGHT = 44;
const ACTION_BAR_HEIGHT = 32;
const PANEL_PADDING = 16;
const OUTER_PADDING = 16;
const WINDOW_WIDTH = 696;

export function useWindowSize(results: Ref<SearchResult[]>) {
  watch(
    results,
    async (items) => {
      let height = SEARCH_BAR_HEIGHT + PANEL_PADDING;
      if (items.length > 0) {
        height += items.length * RESULT_ITEM_HEIGHT + ACTION_BAR_HEIGHT;
      }
      height = Math.min(height, 460) + OUTER_PADDING;
      try {
        const win = getCurrentWindow();
        const pos = await win.outerPosition();
        await win.setSize(new LogicalSize(WINDOW_WIDTH, height));
        await win.setPosition(pos);
      } catch (e) {
        console.warn("setSize failed:", e);
      }
    },
    { immediate: true }
  );
}
