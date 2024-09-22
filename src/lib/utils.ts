import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { z } from 'zod';

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

export const i32 = (errorMessage: string) =>
  z
    .number()
    .int('Has to be an integer')
    .min(-2147483648, {
      message: errorMessage,
    })
    .max(2147483647, {
      message: errorMessage,
    });
