import { Film } from '@/lib/types';
import { ColumnDef } from '@tanstack/react-table';
import { DataTable } from '../core/core';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { DropdownMenuItem } from '@/components/ui/dropdown-menu';
import { Pencil2Icon, TrashIcon } from '@radix-ui/react-icons';
import { useNavigate } from 'react-router-dom';
import { ActionCell } from '../core/cells/actions';
import { useDeleteFilmDialog } from '@/films/FilmCard/tabs/InfoTab/actions/useDeleteFilm';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';

export default function FilmTable() {
  const navigate = useNavigate();
  const { deleteFilm } = useDeleteFilmDialog();

  const columns: ColumnDef<Film>[] = [
    {
      accessorKey: 'id',
      header: 'ID',
    },
    {
      accessorKey: 'title',
      header: 'Title',
    },
    {
      accessorKey: 'release_date',
      header: 'Release Date',
    },
    {
      accessorKey: 'run_time',
      header: 'Runtime',
      cell: ({ row }) => {
        const runTime = row.getValue('run_time');

        if (typeof runTime !== 'number') return '-';

        // Convert seconds to HH:MM:SS
        const hours = Math.floor(runTime / 3600);
        const minutes = Math.floor((runTime % 3600) / 60);
        const seconds = runTime % 60;

        // Display format: HHh MMm
        const displayTime = `${hours}h ${minutes}m`;

        // Tooltip format: HH:MM:SS
        const tooltipTime = `${String(hours).padStart(2, '0')}:${String(
          minutes
        ).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;

        return (
          <Tooltip>
            <TooltipTrigger>
              <div>{displayTime}</div>
            </TooltipTrigger>
            <TooltipContent>
              <div>{tooltipTime}</div>
            </TooltipContent>
          </Tooltip>
        );
      },
    },
    {
      accessorKey: 'rating',
      header: 'Rating',
      cell: ({ row }) => {
        return <p>{row.getValue('rating')}</p>;
      },
    },
    {
      header: 'Actions',
      cell: ({ row }) => (
        <ActionCell>
          <DropdownMenuItem
            onClick={() => navigate(`/film/card/${row.getValue('id')}`)}
          >
            <Pencil2Icon className='w-5 h-5 mr-2' />
            Edit
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => deleteFilm(row.original)}>
            <TrashIcon className='w-5 h-5 mr-2' />
            Delete
          </DropdownMenuItem>
        </ActionCell>
      ),
    },
  ];

  const { data, isLoading, isError, error } = useQuery<Film[], Error>({
    queryKey: ['films'],
    queryFn: async () => {
      const data = await invoke<Film[]>('get_all_films');
      return data || [];
    },
  });

  if (isLoading)
    return (
      <Alert className='px-5'>
        <AlertTitle>Loading</AlertTitle>
        <AlertDescription>Loading your films</AlertDescription>
      </Alert>
    );

  if (isError) {
    console.error(error);
    return (
      <Alert variant='destructive' className='px-5'>
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>Failed to load films</AlertDescription>
      </Alert>
    );
  }

  if (!data || !data.length)
    return (
      <Alert className='px-5'>
        <AlertTitle>No films</AlertTitle>
        <AlertDescription>You have not added any films yet</AlertDescription>
      </Alert>
    );

  return <DataTable columns={columns} data={data} />;
}
