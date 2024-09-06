import { invoke } from '@tauri-apps/api/tauri';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { CheckIcon, Cross2Icon, Pencil2Icon } from '@radix-ui/react-icons';

import { Link } from 'react-router-dom';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { Skeleton } from '@/components/ui/skeleton';
import { Button } from '@/components/ui/button';
import { Film } from '@/lib/types';

function Films() {
  const queryClient = useQueryClient();

  async function syncNewFilms() {
    try {
      // Invoke the Tauri command to sync new films
      await invoke('sync_new_films');
      // Invalidate the films query to refetch the data
      queryClient.invalidateQueries({ queryKey: ['films'] });
    } catch (error) {
      console.error('Error syncing films:', error);
    }
  }

  const {
    data: films,
    error,
    isError,
    isLoading,
  } = useQuery<Film[], Error>({
    queryKey: ['films'],
    queryFn: async () => {
      const data = await invoke<Film[]>('get_all_films');
      return data || [];
    },
  });

  if (isError) return <div>{error.message}</div>;

  return (
    <>
      <div className='pt-12 p-4'>
        <Button onClick={syncNewFilms}>Sync new films</Button>
      </div>
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
          {isLoading ? (
            <TableRow>
              <TableCell>
                <Skeleton />
              </TableCell>
              <TableCell>
                <Skeleton />
              </TableCell>
              <TableCell>
                <Skeleton />
              </TableCell>
              <TableCell>
                <Skeleton />
              </TableCell>
            </TableRow>
          ) : films ? (
            films.map((film) => (
              <TableRow>
                <TableCell>{film.directory.id}</TableCell>
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
            ))
          ) : (
            <TableRow>
              <TableCell>No films found</TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </>
  );
}

function CheckBox({ state }: { state: boolean }) {
  return state ? (
    <CheckIcon className='text-green-700 w-8 h-8' />
  ) : (
    <Cross2Icon className='text-red-700 w-8 h-8' />
  );
}

export default Films;
