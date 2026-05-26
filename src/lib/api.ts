import { convertFileSrc, invoke } from '@tauri-apps/api/core';
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
  provider: string | null;
  provider_id: string | null;
  rating: number | null;
  genres: string | null;
  top_cast: string | null;
  runtime_minutes: number | null;
  metadata_synced_at: number | null;
  metadata_locked: number;
}

export interface Show {
  id: number;
  library_id: number;
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
  provider: string | null;
  provider_id: string | null;
  rating: number | null;
  genres: string | null;
  top_cast: string | null;
  first_air_date: string | null;
  metadata_synced_at: number | null;
  metadata_locked: number;
}

export interface PlayResult {
  session_id: number;
}

export interface MetadataStatusCounts {
  pending: number;
  failed: number;
  tmdb_auth_required: number;
  no_provider_available: number;
  dead_letter: number;
  needs_review: number;
}

export interface MatchCandidate {
  provider: 'tmdb' | 'imdb';
  provider_id: string;
  title: string;
  year: number | null;
}

export interface NeedsReviewItem {
  kind: 'show' | 'movie';
  id: number;
  title: string;
  year: number | null;
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
    invoke<PlayResult>('play_movie', { id, resume }),
  playEpisode: (id: number, resume?: number) =>
    invoke<PlayResult>('play_episode', { id, resume }),

  updateShowMetadata: (id: number, patch: MetadataPatch) =>
    invoke<Show>('update_show_metadata', { id, ...patch }),
  updateMovieMetadata: (id: number, patch: MetadataPatch) =>
    invoke<Movie>('update_movie_metadata', { id, ...patch }),
  updateEpisodeTitle: (id: number, title: string) =>
    invoke<Episode>('update_episode_title', { id, title }),
  mergeShows: (targetId: number, sourceId: number) =>
    invoke<MergeOutcome>('merge_shows', { targetId, sourceId }),
  deleteShow: (id: number) => invoke<void>('delete_show', { id }),
  setShowPosterFromFile: (id: number, sourcePath: string) =>
    invoke<Show>('set_show_poster_from_file', { id, sourcePath }),
  setMoviePosterFromFile: (id: number, sourcePath: string) =>
    invoke<Movie>('set_movie_poster_from_file', { id, sourcePath }),
  resetShowPoster: (id: number) => invoke<Show>('reset_show_poster', { id }),
  resetMoviePoster: (id: number) => invoke<Movie>('reset_movie_poster', { id }),

  metadataStatusCounts: () =>
    invoke<MetadataStatusCounts>('metadata_status_counts'),
  refreshMetadata: (kind: 'show' | 'movie', id: number) =>
    invoke<void>('refresh_metadata', { kind, id }),
  unlinkMetadata: (kind: 'show' | 'movie', id: number) =>
    invoke<void>('unlink_metadata', { kind, id }),
  metadataSearch: (
    kind: 'show' | 'movie',
    query: string,
    year: number | null,
    provider: 'tmdb' | 'imdb',
  ) =>
    invoke<MatchCandidate[]>('metadata_search', { kind, query, year, provider }),
  linkMetadata: (
    kind: 'show' | 'movie',
    mediaId: number,
    provider: 'tmdb' | 'imdb',
    providerId: string,
  ) =>
    invoke<void>('link_metadata', { kind, mediaId, provider, providerId }),
  listNeedsReview: () => invoke<NeedsReviewItem[]>('list_needs_review'),

  adminListRows: (table: string, sortColumn?: string, direction?: 'asc' | 'desc') =>
    invoke<Record<string, unknown>[]>('admin_list_rows', {
      table,
      sortColumn: sortColumn ?? null,
      direction: direction ?? null,
    }),
  adminUpdateRow: (
    table: string,
    primaryKeyValues: unknown[],
    patch: Record<string, unknown>,
  ) =>
    invoke<void>('admin_update_row', {
      table,
      primaryKeyValues,
      patch,
    }),
  adminDeleteRows: (table: string, primaryKeys: unknown[][]) =>
    invoke<void>('admin_delete_rows', { table, primaryKeys }),
  adminFkLabel: (table: string, labelColumn: string, pkValue: unknown) =>
    invoke<string | null>('admin_fk_label', {
      table,
      labelColumn,
      pkValue,
    }),

  pickFolder: () =>
    open({
      directory: true,
      multiple: false,
      title: 'Select a media folder',
    }) as Promise<string | null>,
};

export function formatRuntime(seconds: number | null | undefined): string {
  if (!seconds || seconds <= 0) {
    return '';
  }

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

/**
 * Convert an absolute filesystem path into a URL the webview can load
 * (Tauri asset protocol). Bare filenames are rejected — the backend
 * is responsible for resolving those before returning.
 */
export function posterUrl(posterPath: string | null | undefined): string | undefined {
  if (!posterPath) {
    return undefined;
  }

  return convertFileSrc(posterPath);
}

export function progressPct(
  playback: { progress_seconds: number; duration_seconds: number | null },
): number {
  if (!playback.duration_seconds || playback.duration_seconds <= 0) {
    return 0;
  }
  return Math.min(
    100,
    Math.max(0, (playback.progress_seconds / playback.duration_seconds) * 100),
  );
}
