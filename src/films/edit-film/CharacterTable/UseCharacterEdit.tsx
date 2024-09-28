import { useState, useCallback, useEffect } from 'react';

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api';
import { toast } from '@/hooks/use-toast';
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
import { Character, Film } from '@/lib/types';
import { Button } from '@/components/ui/button';
import { CheckIcon, Cross2Icon } from '@radix-ui/react-icons';

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

  const editCharacterMutation = useMutation({
    mutationFn: async (formValues: FormSchema) => {
      await invoke('update_character', {
        filmId: film?.id,
        actor: selectedCharacter?.actor?.id,
        newDescription: formValues.description,
        newActor: formValues.actor,
      });
    },
    onSuccess: () => {
      toast({
        title: 'Character updated',
        description: `${selectedCharacter?.description} was successfully updated`,
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
        title: 'Failed to update the character',
        description: error.message,
      });
    },
  });

  const { reset } = form;

  useEffect(() => {
    reset({
      description: selectedCharacter?.description,
      actor: selectedCharacter?.actor.id,
    });
  }, [selectedCharacter, open]);

  const onSuccess = useCallback((values: FormSchema) => {
    editCharacterMutation.mutate(values);
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
                  <Button
                    type='submit'
                    disabled={editCharacterMutation.isPending}
                  >
                    <CheckIcon className='w-5 h-5 mr-2' />
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

export default useCharacterEdit;
