import { invoke } from '@tauri-apps/api/tauri';
import { useEffect } from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { CheckIcon, Cross2Icon, Pencil2Icon } from '@radix-ui/react-icons';

import { useAtom } from 'jotai';
import { filmAtom } from '@/lib/atoms';
import { Link } from 'react-router-dom';
export interface Film {
  id: number;
  file: string;
  directory: number;
  link?: string;
  title?: string;
  synopsis?: string;
  release_date?: Date;
  duration?: number;
  cover_image?: string;
  registered: boolean;
}

function Films() {
  const [films, setFilms] = useAtom(filmAtom);

  async function getFilms() {
    try {
      const data: Film[] | null = await invoke('get_all_films');
      console.log(data);
      if (data) setFilms(data);
    } catch (error) {
      console.error('Failed to fetch films:', error);
    }
  }

  useEffect(() => {
    getFilms();
  }, []);

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Directory</TableHead>
          <TableHead>Path</TableHead>
          <TableHead>Registered</TableHead>
          <TableHead>Action</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {films.map((film) => (
          <TableRow>
            <TableCell>{film.directory}</TableCell>
            <TableCell>{film.file}</TableCell>
            <TableCell>
              <CheckBox state={film.registered} />
            </TableCell>
            <TableCell>
              <Link to={`/film/edit/${film.id}`}>
                <Pencil2Icon /> Edit
              </Link>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}

function CheckBox({ state }: { state: boolean }) {
  return (
    <>
      {state ? (
        <CheckIcon className='text-green-700 w-6 h-6' />
      ) : (
        <Cross2Icon className='text-red-700 w-6 h-6' />
      )}
    </>
  );
}

export default Films;
