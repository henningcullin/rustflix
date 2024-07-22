import { invoke } from '@tauri-apps/api/tauri';
import { useEffect, useState } from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { CheckIcon, Cross2Icon } from '@radix-ui/react-icons';
import { Button } from '@/components/ui/button';
interface Entry {
  directory: string;
  path: string;
  film?: Film;
}

export interface Film {
  id: number;
  file: string;
  link?: string;
  title?: string;
  release_year?: number;
  duration?: number;
  cover_image?: string;
}

function Films() {
  const [entries, setEntries] = useState<Entry[]>([]);

  async function initialize() {
    try {
      const fileMap = JSON.parse(await invoke('get_all_files'));
      const files: Entry[] = [];

      // Flatten the fileMap to an array of { directory, path } objects
      for (const [directory, filmArray] of Object.entries(fileMap)) {
        if (!Array.isArray(filmArray)) continue;
        for (const film of filmArray) {
          files.push({
            directory,
            path: film,
          });
        }
      }

      const films: Film[] | null = await invoke('get_all_films');

      if (!films?.length) return setEntries(files);

      // Create a map for quick lookup of films by their file property
      const filmMap = new Map<string, Film>();
      for (const film of films) {
        filmMap.set(film.file, film);
      }

      // Combine files and films based on matching path and file properties
      const combinedEntries = files.map((file) => ({
        ...file,
        film: filmMap.get(file.path),
      }));

      setEntries(combinedEntries);
    } catch (error) {
      console.error('Failed to fetch films:', error);
    }
  }

  useEffect(() => {
    initialize();
  }, []);

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Directory</TableHead>
          <TableHead>Path</TableHead>
          <TableHead>Is Registered</TableHead>
          <TableHead>Register</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {entries
          .sort((a, b) => (a.film == null ? -1 : b.film == null ? 1 : 0))
          .map((entry) => (
            <TableRow>
              <TableCell>{entry.directory}</TableCell>
              <TableCell>{entry.path}</TableCell>
              <TableCell>
                <CheckBox state={entry?.film?.id} />
              </TableCell>
              <TableCell>
                <Button>Register</Button>
              </TableCell>
            </TableRow>
          ))}
      </TableBody>
    </Table>
  );
}

function CheckBox({ state }: { state: number | undefined | null }) {
  return (
    <>
      {state !== null && state !== undefined ? (
        <CheckIcon className='text-green-700 w-6 h-6' />
      ) : (
        <Cross2Icon className='text-red-700 w-6 h-6' />
      )}
    </>
  );
}

export default Films;
