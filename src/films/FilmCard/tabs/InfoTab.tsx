import InfoTable, { InfoTableConfig } from '@/components/cards/InfoTable';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { Film } from '@/lib/types';
import { CheckIcon, Cross2Icon } from '@radix-ui/react-icons';

function RuntimeCell({ runTime }: { runTime?: number }) {
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
}

function RatingCell({ rating }: { rating?: number }) {
  if (typeof rating !== 'number') return '-';

  return <div>{rating}</div>;
}

function CheckboxCell({ checked }: { checked: boolean }) {
  return (
    <div>
      {checked ? (
        <CheckIcon className='h-6 w-6 text-green-500' />
      ) : (
        <Cross2Icon className='h-6 w-6 text-red-500' />
      )}
    </div>
  );
}

export default function InfoTab({ film }: { film: Film }) {
  const tableConfig: InfoTableConfig<Film> = {
    infoRows: [
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
        accessorKey: 'release_data',
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
        cell: ({ item }) => (
          <div>
            {item.has_watched ? (
              <CheckIcon className='h-6 w-6 text-green-500' />
            ) : (
              <Cross2Icon className='h-6 w-6 text-red-500' />
            )}
          </div>
        ),
      },
    ],
  };

  return (
    <div className='w-full'>
      <InfoTable item={film} config={tableConfig}></InfoTable>
    </div>
  );
}
