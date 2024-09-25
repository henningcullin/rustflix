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
import PersonBox from '@/components/PersonBox';
import { Film, Person } from '@/lib/types';
import { Button } from '@/components/ui/button';
import { Cross2Icon } from '@radix-ui/react-icons';

function useDirectorEdit(film: Film | undefined) {
  const [open, setOpen] = useState<boolean>(false);
  const [selectedDirector, setSelectedDirector] = useState<Person | null>(null);
  const queryClient = useQueryClient();

  const formSchema = z.object({
    person: i32('A director requires a person'),
  });

  type FormSchema = z.infer<typeof formSchema>;

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      person: undefined,
    },
  });

  const editDirectorMutation = useMutation({
    mutationFn: async (formValues: FormSchema) => {
      await invoke('update_director', {
        filmId: film?.id,
        actor: selectedDirector?.id,
        newActor: formValues.person,
      });
    },
    onError: (error) => {
      console.error(error);
      toast({
        variant: 'destructive',
        title: 'Failed to update the director',
        description: error.message,
      });
    },
    onSuccess: () => {
      toast({
        title: 'Director updated',
        description: `${selectedDirector?.name} was successfully updated`,
      });
      queryClient.invalidateQueries({
        queryKey: ['film', film?.id?.toString()],
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
    },
  });

  const { reset } = form;

  useEffect(() => {
    reset({
      person: selectedDirector?.id,
    });
  }, [selectedDirector, open]);

  const onSuccess = useCallback((values: FormSchema) => {
    editDirectorMutation.mutate(values);
  }, []);

  const directorEdit = useCallback((director: Person) => {
    setSelectedDirector(director);
    setOpen(true);
  }, []);

  const EditDialog = useCallback(() => {
    return (
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogOverlay>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Edit Director</DialogTitle>
              <DialogDescription>
                Editing {selectedDirector?.name}
              </DialogDescription>
            </DialogHeader>
            <Form {...form}>
              <form onSubmit={form.handleSubmit(onSuccess)}>
                <FormField
                  control={form.control}
                  name='person'
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Person</FormLabel>
                      <FormControl>
                        <PersonBox {...field} />
                      </FormControl>
                      <FormDescription>The director</FormDescription>
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
                    disabled={editDirectorMutation.isPending}
                  >
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
  }, [open, selectedDirector]);

  return { directorEdit, EditDialog };
}

export default useDirectorEdit;
