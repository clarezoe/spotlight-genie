import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const THEME_KEY = "genie-theme";

function resolveTheme(theme: string): string {
  if (theme === "auto") {
    return window.matchMedia("(prefers-color-scheme: light)").matches
      ? "light"
      : "dark";
  }
  return theme;
}

function setThemeOnDom(resolved: string) {
  document.documentElement.setAttribute("data-theme", resolved);
}

const stored = localStorage.getItem(THEME_KEY) || "dark";
const currentTheme = ref(resolveTheme(stored));
setThemeOnDom(currentTheme.value);

function applyTheme(theme: string) {
  const resolved = resolveTheme(theme);
  currentTheme.value = resolved;
  localStorage.setItem(THEME_KEY, theme);
  setThemeOnDom(resolved);
}

export function useTheme() {
  async function init() {
    try {
      const s = await invoke<{ theme: string }>("get_settings");
      applyTheme(s.theme);
    } catch {
      // NOTE: keep current theme from localStorage
    }
  }
  return { currentTheme, applyTheme, init };
}
