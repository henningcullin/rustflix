import type { Theme } from "$lib/types";

let current = $state<Theme>("system");
let mediaQuery: MediaQueryList | null = null;
let mediaListener: ((e: MediaQueryListEvent) => void) | null = null;

function resolveDark(theme: Theme): boolean {
  if (theme === "dark") return true;
  if (theme === "light") return false;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function paint(dark: boolean) {
  const root = document.documentElement;
  if (dark) root.classList.add("dark");
  else root.classList.remove("dark");
}

function detachMedia() {
  if (mediaQuery && mediaListener) {
    mediaQuery.removeEventListener("change", mediaListener);
  }
  mediaQuery = null;
  mediaListener = null;
}

export function applyTheme(theme: Theme) {
  current = theme;
  detachMedia();
  paint(resolveDark(theme));
  if (theme === "system") {
    mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    mediaListener = (e) => paint(e.matches);
    mediaQuery.addEventListener("change", mediaListener);
  }
}

export function currentTheme(): Theme {
  return current;
}
