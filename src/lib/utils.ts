import { appWindow } from '@tauri-apps/api/window';
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

export function isValidDate(dateString: string | undefined) {
  // Check if the input is a string
  if (typeof dateString !== 'string') return false;

  // Attempt to parse the string as a date
  const date = new Date(dateString);

  // Check if the date is invalid (getTime() returns NaN for invalid dates)
  if (isNaN(date.getTime())) return false;

  // Additional validation to check that input string matches the parsed date.
  // Convert the date back to a string and ensure it matches the input.
  const [year, month, day] = dateString.split('-');
  return (
    date.getUTCFullYear() === Number(year) &&
    date.getUTCMonth() === Number(month) - 1 &&
    date.getUTCDate() === Number(day)
  );
}

export async function toggleTauriFullScreen() {
  const isFullscreen = await appWindow.isFullscreen();
  await appWindow.setFullscreen(!isFullscreen); // Toggle
}
