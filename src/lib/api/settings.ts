import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "$lib/types";

export const getSettings = () => invoke<Settings>("get_settings");

export const setTmdbApiKey = (key: string) =>
  invoke<Settings>("set_tmdb_api_key", { key });
