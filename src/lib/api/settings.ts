import { invoke } from "@tauri-apps/api/core";
import type { Settings, Theme } from "$lib/types";

export const getSettings = () => invoke<Settings>("get_settings");

export const setTmdbApiKey = (key: string) =>
  invoke<Settings>("set_tmdb_api_key", { key });

export const setAlias = (alias: string) =>
  invoke<Settings>("set_alias", { alias });

export const setTheme = (theme: Theme) =>
  invoke<Settings>("set_theme", { theme });

export const completeFirstRun = () =>
  invoke<Settings>("complete_first_run");

export const resetFirstRun = () =>
  invoke<Settings>("reset_first_run");
