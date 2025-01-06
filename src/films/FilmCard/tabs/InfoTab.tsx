import InfoTable, { InfoTableConfig } from '@/components/cards/InfoTable';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { Film } from '@/lib/types';

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

export default function InfoTab({ film }: { film: Film }) {
  const tableConfig: InfoTableConfig<Film> = {
    item: film,
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
    ],
  };

  return (
    <div className='w-full'>
      <InfoTable config={tableConfig}></InfoTable>
    </div>
  );
}
