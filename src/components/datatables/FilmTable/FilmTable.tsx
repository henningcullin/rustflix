import { Film } from '@/lib/types';
import { ColumnDef } from '@tanstack/react-table';
import { DataTable } from '../core';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';

const columns: ColumnDef<Film>[] = [
  {
    accessorKey: 'id',
    header: 'ID',
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
