import type { GeniePlugin, SearchResult } from "../../src/types";

const clipboardHistory: { text: string; timestamp: number }[] = [];
const MAX_HISTORY = 50;
let monitorInterval: ReturnType<typeof setInterval> | null = null;
let lastClipText = "";

async function readClipboard(): Promise<string> {
  try {
    return await navigator.clipboard.readText();
  } catch {
    return "";
  }
}

function startMonitor() {
  if (monitorInterval) return;
  monitorInterval = setInterval(async () => {
    const text = await readClipboard();
    if (text && text !== lastClipText) {
      lastClipText = text;
      clipboardHistory.unshift({ text, timestamp: Date.now() });
      if (clipboardHistory.length > MAX_HISTORY) {
        clipboardHistory.pop();
      }
    }
  }, 1000);
}

export const clipboardPlugin: GeniePlugin = {
  id: "integration:clipboard",
  name: "Clipboard History",
  icon: "clipboard",
  keyword: "cb",
  debounceMs: 50,

  async onInit(): Promise<void> {
    startMonitor();
  },

  async onDestroy(): Promise<void> {
    if (monitorInterval) {
      clearInterval(monitorInterval);
      monitorInterval = null;
    }
  },

  async onSearch(query: string): Promise<SearchResult[]> {
    const q = query.toLowerCase().trim();
    const filtered = q
      ? clipboardHistory.filter((e) => e.text.toLowerCase().includes(q))
      : clipboardHistory;

    if (filtered.length === 0) {
      return [
        {
          id: "clipboard:empty",
          title: q ? "No matching clipboard entries" : "Clipboard history is empty",
          subtitle: "Copy something to start tracking",
          category: "CLIP",
          icon: "clipboard",
          action_data: "",
          score: 100,
        },
      ];
    }

    return filtered.slice(0, 8).map((entry, i) => {
      const preview =
        entry.text.length > 60
          ? entry.text.slice(0, 60) + "..."
          : entry.text;
      const ago = formatTimeAgo(entry.timestamp);
      return {
        id: `clipboard:${i}`,
        title: preview.replace(/\n/g, " "),
        subtitle: `Copied ${ago}`,
        category: "CLIP" as const,
        icon: "clipboard",
        action_data: entry.text,
        score: 800 - i,
      };
    });
  },

  async onAction(result: SearchResult): Promise<void> {
    if (result.action_data) {
      await navigator.clipboard.writeText(result.action_data);
    }
  },
};

function formatTimeAgo(ts: number): string {
  const diff = Date.now() - ts;
  const secs = Math.floor(diff / 1000);
  if (secs < 60) return "just now";
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}
