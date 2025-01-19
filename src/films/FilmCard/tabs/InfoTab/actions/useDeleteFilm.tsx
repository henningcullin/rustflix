// DeleteFilmContext.tsx
import React, { createContext, useContext, useState, useCallback } from 'react';
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

const DeleteFilmContext = createContext<DeleteFilmContextProps | undefined>(
  undefined
);

export const DeleteFilmProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [open, setOpen] = useState<boolean>(false);
  const [selectedFilm, setSelectedFilm] = useState<Film | undefined>();

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
        description: (error as Error).message,
      });
    },
  });

  const handleDelete = useCallback(async () => {
    if (selectedFilm && !deleteFilmMutation.isPending) {
      deleteFilmMutation.mutate(selectedFilm);
    }
  }, [selectedFilm, deleteFilmMutation]);

  const deleteFilm = (film: Film) => {
    if (!deleteFilmMutation.isPending) {
      setSelectedFilm(film);
      setOpen(true);
    }
  };

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
            <Button variant='destructive' onClick={handleDelete}>
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
  const context = useContext(DeleteFilmContext);
  if (!context) {
    throw new Error(
      'useDeleteFilmDialog must be used within a DeleteFilmProvider'
    );
  }
  return context;
};
