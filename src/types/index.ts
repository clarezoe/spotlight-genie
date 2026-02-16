export interface SearchResult {
  id: string;
  title: string;
  subtitle: string;
  category: ResultCategory;
  icon: string;
  action_data: string;
  score: number;
}

export type ResultCategory =
  | "APP"
  | "FILE"
  | "CALC"
  | "WEB"
  | "SYS"
  | "SPOTIFY"
  | "CURRENCY"
  | "CLIP";

export interface GeniePlugin {
  id: string;
  name: string;
  icon: string;
  keyword?: string;
  debounceMs?: number;
  onSearch(query: string): Promise<SearchResult[]>;
  onAction(result: SearchResult): Promise<void>;
  onInit?(): Promise<void>;
  onDestroy?(): Promise<void>;
}

export interface PluginKeywordMatch {
  plugin: GeniePlugin;
  query: string;
}

export const KEYWORD_BADGE_COLORS: Record<string, string> = {
  sp: "#1DB954",
  cc: "#3B82F6",
  cb: "#8B5CF6",
};
