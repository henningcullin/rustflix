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

export const i32 = z
  .number()
  .int('Has to be an integer')
  .min(-2147483648, {
    message: 'Value must be greater than or equal to -2,147,483,648.',
  })
  .max(2147483647, {
    message: 'Value must be less than or equal to 2,147,483,647.',
  });
