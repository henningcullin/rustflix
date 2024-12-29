import { useState, useCallback } from 'react';

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api';
import { useToast } from '@/lib/hooks/use-toast';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Film, Person } from '@/lib/types';
import { Button } from '@/components/ui/button';
import { Cross2Icon, TrashIcon } from '@radix-ui/react-icons';

function useDirectorDelete(film: Film | undefined) {
  const [open, setOpen] = useState<boolean>(false);
  const [selectedDirector, setSelectedDirector] = useState<Person | null>(null);

  const { toast } = useToast();
  const queryClient = useQueryClient();

  const deleteDirectorMutation = useMutation({
    mutationFn: async (director: Person) => {
      await invoke('delete_director', {
        filmId: film?.id,
        personId: director.id,
      });
    },
    onSuccess: () => {
      toast({
        title: 'Director deleted',
        description: `${selectedDirector?.name} was successfully removed from ${film?.title}`,
      });
      queryClient.invalidateQueries({
        queryKey: ['film', film?.id?.toString()],
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

  const handleDelete = useCallback(() => {
    if (selectedDirector) {
      deleteDirectorMutation.mutate(selectedDirector);
      setOpen(false);
    }
  }, [selectedDirector, deleteDirectorMutation]);

  const directorDelete = useCallback((director: Person) => {
    setSelectedDirector(director);
    setOpen(true);
  }, []);

  const DeleteDialog = useCallback(() => {
    return (
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Confirm Deletion</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete {selectedDirector?.name}? This
              action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <DialogClose>
              <Button type='button' variant='secondary'>
                <Cross2Icon className='w-5 h-5 mr-2' />
                Cancel
              </Button>
            </DialogClose>
            <Button
              variant='destructive'
              onClick={() => {
                handleDelete();
              }}
            >
              <TrashIcon className='w-5 h-5 mr-2' />
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    );
  }, [open, selectedDirector, handleDelete]);

  return { directorDelete, DeleteDialog };
}

export default useDirectorDelete;
