import { invoke } from "@tauri-apps/api/core";
import type { FilmDetail, FilmListItem } from "$lib/types";

export const listFilms = () => invoke<FilmListItem[]>("list_films");

export const getFilm = (id: number) => invoke<FilmDetail>("get_film", { id });

export const deleteFilm = (id: number) => invoke<void>("delete_film", { id });

export const setLeftOffPoint = (id: number, seconds: number) =>
  invoke<void>("set_left_off_point", { id, seconds });

export const streamUrl = (id: number) =>
  // Tauri v2 rewrites custom schemes to `http://<scheme>.localhost/...` on Windows
  // and `<scheme>://localhost/...` on macOS/Linux. The <video> element handles
  // both via the CSP media-src rule, but we use the consistent URL that the
  // webview accepts on every platform.
  `${window.location.protocol === "https:" ? "https" : "http"}://stream.localhost/film/${id}`;
