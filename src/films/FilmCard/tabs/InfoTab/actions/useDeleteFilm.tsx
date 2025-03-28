// DeleteFilmContext.tsx
import React from 'react';
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
import { toast } from '@/lib/hooks/use-toast';
import { Film } from '@/lib/types';
import { Cross2Icon, TrashIcon } from '@radix-ui/react-icons';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';

// Context Types
type DeleteFilmContextProps = {
  deleteFilm: (film: Film) => void;
};

const DeleteFilmContext = React.createContext<
  DeleteFilmContextProps | undefined
>(undefined);

export const DeleteFilmProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [open, setOpen] = React.useState<boolean>(false);
  const [selectedFilm, setSelectedFilm] = React.useState<Film | null>(null);

  const queryClient = useQueryClient();

  const deleteFilmMutation = useMutation({
    mutationFn: async (film: Film) => {
      await invoke('delete_film', { id: film.id });
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
        title: 'Failed to delete the film',
        description: error.message,
      });
    },
  });

  const isDeleteAllowed = selectedFilm && !deleteFilmMutation.isPending;

  const handleDelete = React.useCallback(async () => {
    if (isDeleteAllowed) {
      deleteFilmMutation.mutate(selectedFilm);
    }
  }, [selectedFilm, deleteFilmMutation]);

  const deleteFilm = React.useCallback(
    (film: Film) => {
      if (!deleteFilmMutation.isPending) {
        setSelectedFilm(film);
        setOpen(true);
      }
    },
    [deleteFilmMutation]
  );

  return (
    <DeleteFilmContext.Provider value={{ deleteFilm }}>
      {children}
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
            <DialogClose asChild>
              <Button type='button' variant='outline'>
                <Cross2Icon className='w-5 h-5 mr-2' />
                Cancel
              </Button>
            </DialogClose>
            <Button
              variant='destructive'
              onClick={handleDelete}
              disabled={!isDeleteAllowed}
            >
              <TrashIcon className='w-5 h-5 mr-2' />
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </DeleteFilmContext.Provider>
  );
};

// Custom Hook for Consuming Context
export const useDeleteFilmDialog = (): DeleteFilmContextProps => {
  const context = React.useContext(DeleteFilmContext);
  if (!context) {
    throw new Error(
      'useDeleteFilmDialog must be used within a DeleteFilmProvider'
    );
  }
  return context;
};
