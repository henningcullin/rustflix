import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * Sanitze filenames in attempt to get a search term for the film
 */
export function getFilmName(filePath: string | undefined): string {
  const fileName = filePath?.split('\\')?.pop();
  const value =
    fileName?.replace(/\.\d{4}.*\.(mp4|mkv)$/, '')?.replaceAll('.', ' ') ?? '';
  return value;
}
