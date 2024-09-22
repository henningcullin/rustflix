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
import { useState, useCallback, useEffect } from 'react';

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
  DialogOverlay,
  DialogTitle,
} from '@/components/ui/dialog';
import SaveCartridgeIcon from '@/components/icons/SaveCartridgeIcon';
import { z } from 'zod';
import { i32 } from '@/lib/utils';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import PersonBox from '@/components/PersonBox';

// Encapsulated Delete Logic Hook
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

function useCharacterEdit(film: Film | undefined) {
  const [open, setOpen] = useState<boolean>(false);
  const [selectedCharacter, setSelectedCharacter] = useState<Character | null>(
    null
  );
  const queryClient = useQueryClient();

  const formSchema = z.object({
    description: z.string().min(1, 'A character requires a description'),
    actor: i32('A character requires an actor'),
  });

  type FormSchema = z.infer<typeof formSchema>;

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      description: '',
      actor: undefined,
    },
  });

  const { reset } = form;

  useEffect(() => {
    reset({
      description: selectedCharacter?.description,
      actor: selectedCharacter?.actor.id,
    });
  }, [selectedCharacter]);

  const onSuccess = useCallback((values: FormSchema) => {
    console.log(values);
  }, []);

  const characterEdit = useCallback((character: Character) => {
    setSelectedCharacter(character);
    setOpen(true);
  }, []);

  const EditDialog = useCallback(() => {
    return (
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogOverlay>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Edit Character</DialogTitle>
              <DialogDescription>
                Editing {selectedCharacter?.description}
              </DialogDescription>
            </DialogHeader>
            <Form {...form}>
              <form onSubmit={form.handleSubmit(onSuccess)}>
                <FormField
                  control={form.control}
                  name='description'
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Description</FormLabel>
                      <FormControl>
                        <Input {...field} />
                      </FormControl>
                      <FormDescription>
                        The description of the character
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name='actor'
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Actor</FormLabel>
                      <FormControl>
                        <PersonBox {...field} />
                      </FormControl>
                      <FormDescription>
                        The actor playing the character
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <DialogFooter>
                  <DialogClose>
                    <Button variant='secondary'>
                      <Cross2Icon className='w-5 h-5 mr-2' />
                      Cancel
                    </Button>
                  </DialogClose>
                  <Button type='submit'>
                    <SaveCartridgeIcon className='w-5 h-5 mr-2' />
                    Save
                  </Button>
                </DialogFooter>
              </form>
            </Form>
          </DialogContent>
        </DialogOverlay>
      </Dialog>
    );
  }, [open, selectedCharacter]);

  return { characterEdit, EditDialog };
}

function CharacterTable({ film }: { film: Film | undefined }) {
  // Use the custom hook for character deletion
  const { characterDelete, DeleteDialog } = useCharacterDelete(film);
  const { characterEdit, EditDialog } = useCharacterEdit(film);

  return (
    <Table>
      <DeleteDialog />
      <EditDialog />
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
                    <DropdownMenuItem onClick={() => characterEdit(character)}>
                      <Pencil2Icon className='w-5 h-5 mr-2' />
                      Edit
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      onClick={() => characterDelete(character)}
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
  );
}

export default CharacterTable;
