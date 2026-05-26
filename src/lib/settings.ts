import { invoke } from '@tauri-apps/api/core';

export type MetadataMode =
  | 'off'
  | 'tmdb_only'
  | 'imdb_only'
  | 'prefer_tmdb'
  | 'prefer_imdb';

type SettingDef<T> = {
  default: T;
  parse: (raw: string | null) => T;
  encode: (value: T) => string | null;
};

export const SETTINGS = {
  tmdb_api_key: {
    default: null as string | null,
    parse: (raw: string | null): string | null => raw,
    encode: (value: string | null): string | null => value,
  } satisfies SettingDef<string | null>,

  metadata_mode: {
    default: 'prefer_tmdb' as MetadataMode,
    parse: (raw: string | null): MetadataMode => {
      const valid: readonly MetadataMode[] = [
        'off',
        'tmdb_only',
        'imdb_only',
        'prefer_tmdb',
        'prefer_imdb',
      ];
      if (raw === null || raw === undefined) {
        return 'prefer_tmdb';
      }
      if (!(valid as readonly string[]).includes(raw)) {
        console.warn(`metadata_mode: invalid value "${raw}", falling back to default`);
        return 'prefer_tmdb';
      }
      return raw as MetadataMode;
    },
    encode: (value: MetadataMode): string => value,
  } satisfies SettingDef<MetadataMode>,

  scrape_language: {
    default: 'en',
    parse: (raw: string | null): string => raw ?? 'en',
    encode: (value: string): string => value,
  } satisfies SettingDef<string>,
} as const;

export type SettingKey = keyof typeof SETTINGS;
export type SettingValue<K extends SettingKey> = (typeof SETTINGS)[K]['default'];

export async function getSetting<K extends SettingKey>(key: K): Promise<SettingValue<K>> {
  const raw = await invoke<string | null>('get_app_setting', { key });
  return SETTINGS[key].parse(raw) as SettingValue<K>;
}

export async function setSetting<K extends SettingKey>(
  key: K,
  value: SettingValue<K>,
): Promise<void> {
  const encoded = SETTINGS[key].encode(value as never);
  await invoke<void>('set_app_setting', { key, value: encoded });
}
