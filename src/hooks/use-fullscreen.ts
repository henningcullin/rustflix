import { appWindow } from '@tauri-apps/api/window';
import { useEffect } from 'react';

export default function useFullscreen(): void {
  useEffect(() => {
    // Define the type for the keyboard event
    const handleKeyDown = async (event: KeyboardEvent): Promise<void> => {
      if (event.key === 'F11') {
        event.preventDefault(); // Prevent the default F11 behavior

        const isFullscreen = await appWindow.isFullscreen();
        await appWindow.setFullscreen(!isFullscreen); // Toggle fullscreen
      }
    };

    // Attach the event listener on mount
    window.addEventListener('keydown', handleKeyDown);

    // Clean up the event listener on unmount
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, []);
}
