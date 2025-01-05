import { Film } from '@/lib/types';
import { ColumnDef } from '@tanstack/react-table';
import { DataTable } from '../core';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';

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
];

export default function FilmTable() {
  const { data, isLoading, isError, error } = useQuery<Film[], Error>({
    queryKey: ['films'],
    queryFn: async () => {
      const data = await invoke<Film[]>('get_all_films');
      return data || [];
    },
  });

  if (isLoading) return <div>loading</div>;

  if (isError) {
    console.error(error);
    return <div>error</div>;
  }

  if (!data) return <div>no films</div>;

  return (
    <div className='container mx-auto py-10'>
      <DataTable columns={columns} data={data} />
    </div>
  );
}
