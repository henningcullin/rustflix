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

// Only `shows` is configured for now — the other six configs land in the
// admin PR 3 alongside the drawer + FK chips. Adding a new table here
// before then will work but it'll render with default behaviour only.
export const TABLES: Partial<Record<TableId, TableConfig>> = {
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
