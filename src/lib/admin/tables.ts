export type TableId =
  | 'libraries'
  | 'shows'
  | 'movies'
  | 'episodes'
  | 'watch_history'
  | 'metadata_jobs'
  | 'app_settings';

export interface ColumnConfig {
  key: string;
  label?: string;
  readonly?: boolean;
  hideInGrid?: boolean;
  kind?: 'text' | 'json' | 'datetime' | 'boolean';
  fkTable?: TableId;
  fkLabel?: string;
}

export interface TableConfig {
  id: TableId;
  label: string;
  primaryKey: string[];
  defaultSort: { column: string; direction: 'asc' | 'desc' };
  columns: ColumnConfig[];
}

export const TABLES: Record<TableId, TableConfig> = {
  libraries: {
    id: 'libraries',
    label: 'Libraries',
    primaryKey: ['id'],
    defaultSort: { column: 'id', direction: 'asc' },
    columns: [
      { key: 'id', readonly: true },
      { key: 'path' },
      { key: 'kind' },
      { key: 'added_at', kind: 'datetime', readonly: true },
    ],
  },
  shows: {
    id: 'shows',
    label: 'Shows',
    primaryKey: ['id'],
    defaultSort: { column: 'title', direction: 'asc' },
    columns: [
      { key: 'id', readonly: true },
      { key: 'title' },
      { key: 'year' },
      { key: 'library_id', fkTable: 'libraries', fkLabel: 'path' },
      { key: 'provider' },
      { key: 'rating', readonly: true },
      { key: 'metadata_locked', kind: 'boolean' },
      { key: 'genres', kind: 'json', hideInGrid: true },
      { key: 'top_cast', kind: 'json', hideInGrid: true },
      { key: 'overview', kind: 'text', hideInGrid: true },
      { key: 'folder_path' },
      { key: 'fingerprint', readonly: true },
      { key: 'added_at', kind: 'datetime', readonly: true },
    ],
  },
  movies: {
    id: 'movies',
    label: 'Movies',
    primaryKey: ['id'],
    defaultSort: { column: 'title', direction: 'asc' },
    columns: [
      { key: 'id', readonly: true },
      { key: 'title' },
      { key: 'year' },
      { key: 'library_id', fkTable: 'libraries', fkLabel: 'path' },
      { key: 'provider' },
      { key: 'rating', readonly: true },
      { key: 'runtime_minutes' },
      { key: 'metadata_locked', kind: 'boolean' },
      { key: 'genres', kind: 'json', hideInGrid: true },
      { key: 'top_cast', kind: 'json', hideInGrid: true },
      { key: 'overview', kind: 'text', hideInGrid: true },
      { key: 'path', hideInGrid: true },
      { key: 'added_at', kind: 'datetime', readonly: true },
    ],
  },
  episodes: {
    id: 'episodes',
    label: 'Episodes',
    primaryKey: ['id'],
    defaultSort: { column: 'show_id', direction: 'asc' },
    columns: [
      { key: 'id', readonly: true },
      { key: 'show_id', fkTable: 'shows', fkLabel: 'title' },
      { key: 'season' },
      { key: 'episode' },
      { key: 'title' },
      { key: 'duration_seconds' },
      { key: 'path', hideInGrid: true },
      { key: 'added_at', kind: 'datetime', readonly: true },
    ],
  },
  watch_history: {
    id: 'watch_history',
    label: 'Watch history',
    primaryKey: ['media_kind', 'media_id'],
    defaultSort: { column: 'last_watched_at', direction: 'desc' },
    columns: [
      { key: 'media_kind' },
      { key: 'media_id' },
      { key: 'progress_seconds' },
      { key: 'duration_seconds' },
      { key: 'watched', kind: 'boolean' },
      { key: 'last_watched_at', kind: 'datetime' },
    ],
  },
  metadata_jobs: {
    id: 'metadata_jobs',
    label: 'Metadata jobs',
    primaryKey: ['kind', 'media_id'],
    defaultSort: { column: 'enqueued_at', direction: 'desc' },
    columns: [
      { key: 'kind' },
      { key: 'media_id' },
      { key: 'enqueued_at', kind: 'datetime', readonly: true },
      { key: 'attempts' },
      { key: 'last_error', hideInGrid: true },
      { key: 'next_attempt_at', kind: 'datetime' },
    ],
  },
  app_settings: {
    id: 'app_settings',
    label: 'App settings',
    primaryKey: ['key'],
    defaultSort: { column: 'key', direction: 'asc' },
    columns: [
      { key: 'key' },
      { key: 'value' },
    ],
  },
};

export function humanizeKey(key: string): string {
  return key
    .split('_')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

export function columnLabel(column: ColumnConfig): string {
  if (column.label) {
    return column.label;
  }
  return humanizeKey(column.key);
}
