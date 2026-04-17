import { invoke } from "@tauri-apps/api/core";
import type { TmdbSearchResult } from "$lib/types";

export const tmdbSearch = (query: string) =>
  invoke<TmdbSearchResult[]>("tmdb_search", { query });

export const tmdbImportFilm = (filePath: string, tmdbId: number) =>
  invoke<number>("tmdb_import_film", { filePath, tmdbId });
