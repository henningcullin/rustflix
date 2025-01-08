import InfoTable, { InfoTableConfig } from '@/components/cards/InfoTable';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { toast } from '@/lib/hooks/use-toast';
import { Film } from '@/lib/types';
import {
  CheckIcon,
  Cross2Icon,
  ExternalLinkIcon,
  TrashIcon,
} from '@radix-ui/react-icons';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { shell } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/tauri';
import React from 'react';

function useDeleteFilm() {
  const [open, setOpen] = React.useState<boolean>(false);
  const [selectedFilm, setSelectedFilm] = React.useState<Film | undefined>();

  const queryClient = useQueryClient();

  const deleteFilmMutation = useMutation({
    mutationFn: async (film: Film) => {
      await invoke('delete_film', {
        id: film.id,
      });
    },
    onSuccess: () => {
      toast({
        title: 'Film deleted',
        description: `${selectedFilm?.title} was successfully removed`,
      });

      setOpen(false);

      queryClient.invalidateQueries({
        queryKey: ['film', selectedFilm?.id?.toString()],
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
    },
    onError: (error) => {
      console.error(error);

      toast({
        variant: 'destructive',
        title: 'Failed to delete the director',
        description: error.message,
      });
    },
  });

  const handleDelete = React.useCallback(async () => {
    if (selectedFilm && !deleteFilmMutation.isPending) {
      deleteFilmMutation.mutate(selectedFilm);
    }
  }, [selectedFilm, deleteFilmMutation]);

  const DeleteFilmDialog = React.useCallback(
    () => (
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Confirm deletion</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete {selectedFilm?.title}?<br />
              This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <DialogClose>
              <Button type='button' variant='outline'>
                <Cross2Icon className='w-5 h-5 mr-2' />
                Cancel
              </Button>
            </DialogClose>
            <Button variant='destructive' onClick={handleDelete}>
              <TrashIcon className='w-5 h-5 mr-2' />
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    ),
    [selectedFilm, handleDelete]
  );

  function deleteFilm(film: Film) {
    if (!deleteFilmMutation.isPending) {
      setSelectedFilm(film);

      setOpen(true);
    }
  }

  return { DeleteFilmDialog, deleteFilm };
}

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

function IMDBIDCell({ imdb_id }: { imdb_id?: string }) {
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
  const { DeleteFilmDialog, deleteFilm } = useDeleteFilm();

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
      <DeleteFilmDialog />
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
