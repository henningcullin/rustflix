import Avatar from '@/components/Avatar';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from '@/components/ui/hover-card';
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Character, Film } from '@/lib/types';
import {
  Cross2Icon,
  DotsHorizontalIcon,
  EyeOpenIcon,
  Pencil2Icon,
  TrashIcon,
} from '@radix-ui/react-icons';
import { useCallback, useState } from 'react';

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api';
import { useToast } from '@/hooks/use-toast';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import SaveCartridgeIcon from '@/components/icons/SaveCartridgeIcon';

function CharacterTable({ film }: { film: Film | undefined }) {
  const [isDeleteOpen, setisDeleteOpen] = useState<boolean>(false);
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
      setisDeleteOpen(false);
    }
  }, [selectedCharacter, deleteCharacterMutation]);

  const openDeleteConfirm = (character: Character) => {
    setSelectedCharacter(character);
    setisDeleteOpen(true);
  };

  return (
    <>
      <Dialog open={isDeleteOpen} onOpenChange={setisDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Confirm Deletion</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete {selectedCharacter?.description}?
              This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant='outline' onClick={() => setisDeleteOpen(false)}>
              <Cross2Icon className='w-5 h-5 mr-2' />
              Cancel
            </Button>
            <Button
              variant='destructive'
              onClick={() => {
                handleDelete();
                setisDeleteOpen(false);
              }}
            >
              <TrashIcon className='w-5 h-5 mr-2' />
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <Table>
        <TableCaption>List of all characters in the film</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead>Avatar</TableHead>
            <TableHead>Description</TableHead>
            <TableHead>Actor</TableHead>
            <TableHead className='w-12'>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {film?.stars?.map((character) => (
            <TableRow key={character.actor.id}>
              <TableCell>
                <Avatar id={character.actor.id}></Avatar>
              </TableCell>
              <TableCell>{character.description}</TableCell>
              <TableCell>
                <HoverCard>
                  <HoverCardTrigger>
                    <span className='hover:underline underline-offset-4 cursor-pointer'>
                      {character.actor.name}
                    </span>
                  </HoverCardTrigger>
                  <HoverCardContent>
                    <b>{character.actor.id}</b>
                    <br />
                    <i>{character.actor.imdb_id}</i>
                  </HoverCardContent>
                </HoverCard>
              </TableCell>
              <TableCell>
                <DropdownMenu>
                  <DropdownMenuTrigger>
                    <Button variant='outline' className='w-10 h-10 p-0'>
                      <DotsHorizontalIcon />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent>
                    <DropdownMenuLabel>Actions</DropdownMenuLabel>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem>
                      <EyeOpenIcon className='w-5 h-5 mr-2' />
                      View
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuGroup>
                      <DropdownMenuItem>
                        <Pencil2Icon className='w-5 h-5 mr-2' />
                        Edit
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => openDeleteConfirm(character)}
                      >
                        <TrashIcon className='w-5 h-5 mr-2' />
                        Delete
                      </DropdownMenuItem>
                    </DropdownMenuGroup>
                  </DropdownMenuContent>
                </DropdownMenu>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </>
  );
}

export default CharacterTable;
