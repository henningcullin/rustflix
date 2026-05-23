import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export type LibraryKind = 'movies' | 'series' | 'mixed';

export interface Library {
  id: number;
  path: string;
  kind: LibraryKind;
}

export type PosterOrigin = 'auto' | 'manual';

export interface Movie {
  id: number;
  title: string;
  year: number | null;
  path: string;
  poster_path: string | null;
  poster_origin: PosterOrigin | null;
  overview: string | null;
  duration_seconds: number | null;
  progress_seconds: number;
  watched: boolean;
  added_at: number;
}

export interface Show {
  id: number;
  title: string;
  year: number | null;
  folder_path: string;
  fingerprint: string;
  poster_path: string | null;
  poster_origin: PosterOrigin | null;
  overview: string | null;
  episode_count: number;
  watched_count: number;
  added_at: number;
}

export interface Episode {
  id: number;
  show_id: number;
  season: number;
  episode: number;
  title: string;
  path: string;
  duration_seconds: number | null;
  progress_seconds: number;
  watched: boolean;
}

export interface Season {
  season: number;
  episodes: Episode[];
}

export type ContinueWatchingItem =
  | { kind: 'movie'; movie: Movie }
  | { kind: 'episode'; show: Show; episode: Episode };

export interface ScanReport {
  libraries_scanned: number;
  movies_added: number;
  episodes_added: number;
  shows_added: number;
}

export interface EpisodeRef {
  season: number;
  episode: number;
}

export interface MergeOutcome {
  conflicts: EpisodeRef[];
}

export interface MetadataPatch {
  title?: string;
  year?: number;
  overview?: string;
}

export function pickImageFile(): Promise<string | null> {
  return open({
    directory: false,
    multiple: false,
    title: 'Select a poster image',
    filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'webp'] }],
  }) as Promise<string | null>;
}

export const api = {
  listLibraries: () => invoke<Library[]>('list_libraries'),
  addLibrary: (path: string, kind: LibraryKind = 'mixed') =>
    invoke<Library>('add_library', { path, kind }),
  removeLibrary: (id: number) => invoke<void>('remove_library', { id }),
  scanLibraries: () => invoke<ScanReport>('scan_libraries'),

  listMovies: () => invoke<Movie[]>('list_movies'),
  getMovie: (id: number) => invoke<Movie>('get_movie', { id }),

  listShows: () => invoke<Show[]>('list_shows'),
  getShow: (id: number) => invoke<Show>('get_show', { id }),
  getSeasons: (showId: number) => invoke<Season[]>('get_seasons', { showId }),
  getEpisode: (id: number) => invoke<Episode>('get_episode', { id }),

  continueWatching: () => invoke<ContinueWatchingItem[]>('continue_watching'),
  setWatched: (kind: 'movie' | 'episode', id: number, watched: boolean) =>
    invoke<void>('set_watched', { kind, id, watched }),

  checkMpv: () => invoke<boolean>('check_mpv'),
  playMovie: (id: number, resume?: number) =>
    invoke<{ session_id: number }>('play_movie', { id, resume }),
  playEpisode: (id: number, resume?: number) =>
    invoke<{ session_id: number }>('play_episode', { id, resume }),

  updateShowMetadata: (id: number, patch: MetadataPatch) =>
    invoke<Show>('update_show_metadata', { id, ...patch }),
  updateMovieMetadata: (id: number, patch: MetadataPatch) =>
    invoke<Movie>('update_movie_metadata', { id, ...patch }),
  mergeShows: (targetId: number, sourceId: number) =>
    invoke<MergeOutcome>('merge_shows', { targetId, sourceId }),
  setShowPosterFromFile: (id: number, sourcePath: string) =>
    invoke<Show>('set_show_poster_from_file', { id, sourcePath }),
  setMoviePosterFromFile: (id: number, sourcePath: string) =>
    invoke<Movie>('set_movie_poster_from_file', { id, sourcePath }),
  resetShowPoster: (id: number) => invoke<Show>('reset_show_poster', { id }),
  resetMoviePoster: (id: number) => invoke<Movie>('reset_movie_poster', { id }),

  pickFolder: () =>
    open({
      directory: true,
      multiple: false,
      title: 'Select a media folder',
    }) as Promise<string | null>,
};

export function formatRuntime(seconds: number | null | undefined): string {
  if (!seconds || seconds <= 0) return '';
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export function progressPct(p: { progress_seconds: number; duration_seconds: number | null }): number {
  if (!p.duration_seconds || p.duration_seconds <= 0) return 0;
  return Math.min(100, Math.max(0, (p.progress_seconds / p.duration_seconds) * 100));
}
