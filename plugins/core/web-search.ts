import { invoke } from "@tauri-apps/api/core";
import type { GeniePlugin, SearchResult } from "../../src/types";

export const webSearchPlugin: GeniePlugin = {
  id: "core:web-search",
  name: "Web Search",
  icon: "globe",

  async onSearch(_query: string): Promise<SearchResult[]> {
    return [];
  },

  async onAction(result: SearchResult): Promise<void> {
    await invoke("launch_item", {
      actionData: result.action_data,
      category: "WEB",
    });
  },
};
