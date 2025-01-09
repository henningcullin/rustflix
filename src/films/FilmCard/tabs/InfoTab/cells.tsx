import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { toast } from '@/lib/hooks/use-toast';
import { CheckIcon, Cross2Icon, ExternalLinkIcon } from '@radix-ui/react-icons';
import { shell } from '@tauri-apps/api';

export function RuntimeCell({ runTime }: { runTime?: number }) {
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
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger>
          <div>{displayTime}</div>
        </TooltipTrigger>
        <TooltipContent>
          <div>{tooltipTime}</div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

export function RatingCell({ rating }: { rating?: number }) {
  if (typeof rating !== 'number') return '-';

  return <div>{rating}</div>;
}

function handleIDMBLink(id: string) {
  shell.open(`https://www.imdb.com/title/${id}/`).catch((error) => {
    toast({
      variant: 'destructive',
      title: 'Failed to open the link',
      description: error.message,
    });
    console.error('Failed to open IMDB link from film', error);
  });
}

export function IMDBIDCell({ imdb_id }: { imdb_id?: string }) {
  if (typeof imdb_id !== 'string' || !imdb_id) return <p>-</p>;

  return (
    <div className='flex items-center justify-between w-full'>
      <p>{imdb_id}</p>
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              className='p-2'
              variant='external-link'
              onClick={() => handleIDMBLink(imdb_id)}
            >
              <ExternalLinkIcon className='h-5 w-5' />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Open in IMDB</TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </div>
  );
}

export function CheckboxCell({ checked }: { checked: boolean }) {
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
