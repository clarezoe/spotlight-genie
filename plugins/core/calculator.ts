import { invoke } from "@tauri-apps/api/core";
import type { GeniePlugin, SearchResult } from "../../src/types";

export const calculatorPlugin: GeniePlugin = {
  id: "core:calculator",
  name: "Calculator",
  icon: "calculator",

  async onSearch(query: string): Promise<SearchResult[]> {
    const hasOp = /[+\-*/^%]/.test(query);
    if (!hasOp) return [];
    const result = await invoke<string | null>("calculate", { expression: query });
    if (!result) return [];
    return [
      {
        id: "calc:result",
        title: result,
        subtitle: "Inline Calculator",
        category: "CALC",
        icon: "calculator",
        action_data: result.split("=").pop()?.trim() ?? result,
        score: 1000,
      },
    ];
  },

  async onAction(result: SearchResult): Promise<void> {
    await navigator.clipboard.writeText(result.action_data);
  },
};
