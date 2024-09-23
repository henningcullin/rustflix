import { useState, useCallback } from 'react';

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api';
import { useToast } from '@/hooks/use-toast';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Character, Film } from '@/lib/types';
import { Button } from '@/components/ui/button';
import { Cross2Icon, TrashIcon } from '@radix-ui/react-icons';

function useCharacterDelete(film: Film | undefined) {
  const [open, setOpen] = useState<boolean>(false);
  const [selectedCharacter, setSelectedCharacter] = useState<Character | null>(
    null
  );
  const { toast } = useToast();
  const queryClient = useQueryClient();

  const deleteCharacterMutation = useMutation({
    mutationFn: async (character: Character) => {
      await invoke('delete_character', {
        filmId: character.film_id,
        actor: character.actor.id,
      });
    },
    onError: (error) => {
      console.error(error);
      toast({
        variant: 'destructive',
        title: 'Failed to delete the character',
        description: error.message,
      });
    },
    onSuccess: () => {
      toast({
        title: 'Character deleted',
        description: `${selectedCharacter?.description} was successfully removed from ${film?.title}`,
      });
      queryClient.invalidateQueries({
        queryKey: ['film', film?.id?.toString()],
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
    },
  });

  const handleDelete = useCallback(() => {
    if (selectedCharacter) {
      deleteCharacterMutation.mutate(selectedCharacter);
      setOpen(false);
    }
  }, [selectedCharacter, deleteCharacterMutation]);

  const characterDelete = useCallback((character: Character) => {
    setSelectedCharacter(character);
    setOpen(true);
  }, []);

  const DeleteDialog = useCallback(() => {
    return (
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Confirm Deletion</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete {selectedCharacter?.description}?
              This action cannot be undone.
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
  }, [open, selectedCharacter, handleDelete]);

  return { characterDelete, DeleteDialog };
}

export default useCharacterDelete;
