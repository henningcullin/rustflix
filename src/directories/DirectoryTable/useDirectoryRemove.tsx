import { useState, useCallback } from 'react';

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api';
import { useToast } from '@/components/hooks/use-toast';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Directory } from '@/components/lib/types';
import { Button } from '@/components/ui/button';
import { Cross2Icon, TrashIcon } from '@radix-ui/react-icons';

function useDirectoryRemove() {
  const [open, setOpen] = useState<boolean>(false);
  const [selectedDirectory, setSelectedDirectory] = useState<Directory | null>(
    null
  );

  const { toast } = useToast();
  const queryClient = useQueryClient();

  const removeDirectoryMutation = useMutation({
    mutationFn: async (directory: Directory) => {
      await invoke('delete_directory', {
        id: directory.id,
      });
    },
    onSuccess: () => {
      toast({
        title: 'Directory removed',
        description: `Directory was successfully removed`,
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
      queryClient.invalidateQueries({ queryKey: ['directories'] });
    },
    onError: (error) => {
      console.error(error);
      toast({
        variant: 'destructive',
        title: 'Failed to remove the directory',
        description: error.message,
      });
    },
  });

  const handleRemove = useCallback(() => {
    if (selectedDirectory) {
      removeDirectoryMutation.mutate(selectedDirectory);
      setOpen(false);
    }
  }, [selectedDirectory, removeDirectoryMutation]);

  const directoryRemove = useCallback((directory: Directory) => {
    setSelectedDirectory(directory);
    setOpen(true);
  }, []);

  const RemoveDialog = useCallback(() => {
    return (
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Confirm Removal</DialogTitle>
            <DialogDescription>
              <p>Are you sure you want to remove this directory?</p>
              <br />
              <p className='text-lg'>"{selectedDirectory?.path}"</p>
              <br />
              <p className='text-destructive'>
                <b>All films from this directory will be permanently removed</b>
              </p>
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <DialogClose>
              <Button type='button' variant='outline'>
                <Cross2Icon className='w-5 h-5 mr-2' />
                Cancel
              </Button>
            </DialogClose>
            <Button
              variant='destructive'
              onClick={() => {
                handleRemove();
              }}
              disabled={removeDirectoryMutation.isPending}
            >
              <TrashIcon className='w-5 h-5 mr-2' />
              {removeDirectoryMutation.isPending ? 'Removing...' : 'Remove'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    );
  }, [open, selectedDirectory, handleRemove]);

  return { directoryRemove, RemoveDialog };
}

export default useDirectoryRemove;
