import { getSettings } from "$lib/api/settings";
import type { Settings } from "$lib/types";

let settings = $state<Settings | null>(null);
let loading = $state(false);

export async function loadSettings(force = false) {
  if (!force && settings !== null) return settings;
  loading = true;
  try {
    settings = await getSettings();
    return settings;
  } finally {
    loading = false;
  }
}

export function setSettings(next: Settings) {
  settings = next;
}

export function current() {
  return settings;
}

export function isLoading() {
  return loading;
}

export function hasTmdbKey() {
  return !!settings?.tmdb_api_key;
}

export function alias() {
  return settings?.alias ?? null;
}

export function theme() {
  return settings?.theme ?? 'system';
}

export function firstRunCompleted() {
  return settings?.first_run_completed ?? false;
}
