import { InfoTable, InfoTableConfig } from '@/components/cards/InfoTable';
import { Button } from '@/components/ui/button';
import { Film } from '@/lib/types';
import { CheckboxCell, IMDBIDCell, RatingCell, RuntimeCell } from './cells';
import { useDeleteFilmDialog } from './actions/useDeleteFilm';

export default function InfoTab({ film }: { film: Film }) {
  const { deleteFilm } = useDeleteFilmDialog();

  const tableConfig: InfoTableConfig<Film> = {
    infoRows: [
      {
        accessorKey: 'id',
        caption: 'Database ID',
        cell: ({ item }) => <p>{item.id}</p>,
      },
      {
        accessorKey: 'imdb_id',
        caption: 'IMDB ID',
        cell: ({ item }) => <IMDBIDCell imdb_id={item.imdb_id} />,
      },
      {
        accessorKey: 'title',
        caption: 'Title',
        cell: ({ item }) => <p>{item.title}</p>,
      },
      {
        accessorKey: 'run_time',
        caption: 'Runtime',
        cell: ({ item }) => <RuntimeCell runTime={item.run_time} />,
      },
      {
        accessorKey: 'rating',
        caption: 'Rating',
        cell: ({ item }) => <RatingCell rating={item.rating} />,
      },
      {
        accessorKey: 'release_date',
        caption: 'Release date',
        cell: ({ item }) => <div>{item.release_date}</div>,
      },
      {
        accessorKey: 'plot',
        caption: 'Plot',
        cell: ({ item }) => <span>{item.plot}</span>,
      },
      {
        accessorKey: 'has_watched',
        caption: 'Watched',
        cell: ({ item }) => <CheckboxCell checked={item.has_watched} />,
      },
      {
        accessorKey: 'has_color',
        caption: 'Color',
        cell: ({ item }) => <CheckboxCell checked={!!item.has_color} />,
      },
    ],
  };

  return (
    <div className='w-full border-ws rounded-sm'>
      <div className='flex place-content-center w-full mb-4'>
        <div className='inline-flex gap-3'>
          <Button>New</Button>
          <Button variant='destructive' onClick={() => deleteFilm(film)}>
            Delete
          </Button>
          <Button>Edit</Button>
        </div>
      </div>
      <InfoTable<Film> item={film} config={tableConfig}></InfoTable>
    </div>
  );
}
