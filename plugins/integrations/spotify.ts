import type { GeniePlugin, SearchResult } from "../../src/types";
import { invoke } from "@tauri-apps/api/core";

const SPOTIFY_COMMANDS = [
  { cmd: "play", title: "Play / Resume", subtitle: "Spotify • Playback", icon: "music" },
  { cmd: "pause", title: "Pause", subtitle: "Spotify • Playback", icon: "music" },
  { cmd: "next", title: "Next Track", subtitle: "Spotify • Playback", icon: "music" },
  { cmd: "prev", title: "Previous Track", subtitle: "Spotify • Playback", icon: "music" },
];

export const spotifyPlugin: GeniePlugin = {
  id: "integration:spotify",
  name: "Spotify",
  icon: "music",
  keyword: "sp",
  debounceMs: 100,

  async onSearch(query: string): Promise<SearchResult[]> {
    const q = query.toLowerCase().trim();
    if (!q) {
      return SPOTIFY_COMMANDS.map((c, i) => ({
        id: `spotify:${c.cmd}`,
        title: c.title,
        subtitle: c.subtitle,
        category: "SPOTIFY" as const,
        icon: c.icon,
        action_data: c.cmd,
        score: 900 - i,
      }));
    }

    const filtered = SPOTIFY_COMMANDS.filter(
      (c) => c.cmd.includes(q) || c.title.toLowerCase().includes(q)
    );

    const results: SearchResult[] = filtered.map((c, i) => ({
      id: `spotify:${c.cmd}`,
      title: c.title,
      subtitle: c.subtitle,
      category: "SPOTIFY" as const,
      icon: c.icon,
      action_data: c.cmd,
      score: 800 - i,
    }));

    if (q.length > 1 && !filtered.length) {
      results.push({
        id: "spotify:search",
        title: `Play: ${query}`,
        subtitle: "Spotify • Search & Play",
        category: "SPOTIFY",
        icon: "music",
        action_data: `search:${query}`,
        score: 700,
      });
    }

    return results;
  },

  async onAction(result: SearchResult): Promise<void> {
    const cmd = result.action_data;
    try {
      if (cmd.startsWith("search:")) {
        const term = cmd.slice(7);
        const url = `https://open.spotify.com/search/${encodeURIComponent(term)}`;
        await invoke("launch_item", { actionData: url, category: "WEB" });
      } else {
        const systemCommandMap: Record<string, string> = {
          play: "spotify_play",
          pause: "spotify_pause",
          next: "spotify_next",
          prev: "spotify_prev",
        };
        const systemCommand = systemCommandMap[cmd];
        if (systemCommand) {
          await invoke("run_system_command", { command: systemCommand }).catch(
            () => {}
          );
        }
      }
    } catch {
      // NOTE: Silently fail — Spotify may not be running
    }
  },
};
