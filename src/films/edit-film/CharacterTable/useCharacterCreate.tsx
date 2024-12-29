import { useState, useCallback } from 'react';

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api';
import { toast } from '@/components/hooks/use-toast';
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
import { Film } from '@/lib/types';
import { Button } from '@/components/ui/button';
import { Cross2Icon, PlusIcon } from '@radix-ui/react-icons';

function useCharacterCreate(film: Film | undefined) {
  const [open, setOpen] = useState<boolean>(false);
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

  const createCharacterMutation = useMutation({
    mutationFn: async (formValues: FormSchema) => {
      await invoke('create_character', {
        filmId: film?.id,
        actorId: formValues.actor,
        description: formValues.description,
      });
    },
    onSuccess: () => {
      setOpen(false);
      toast({
        title: 'Character created',
        description: `Character was successfully created`,
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
        title: 'Failed to create the character',
        description: error.message,
      });
    },
  });

  const { reset } = form;

  const onSuccess = useCallback((values: FormSchema) => {
    createCharacterMutation.mutate(values);
  }, []);

  const characterCreate = useCallback(() => {
    reset({
      description: '',
      actor: undefined,
    });
    setOpen(true);
  }, []);

  const CreateDialog = useCallback(() => {
    return (
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogOverlay>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Create character</DialogTitle>
              <DialogDescription>Creating character</DialogDescription>
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
                    disabled={createCharacterMutation.isPending}
                  >
                    <PlusIcon className='w-5 h-5 mr-2' />
                    Create
                  </Button>
                </DialogFooter>
              </form>
            </Form>
          </DialogContent>
        </DialogOverlay>
      </Dialog>
    );
  }, [open]);

  return { characterCreate, CreateDialog };
}

export default useCharacterCreate;
