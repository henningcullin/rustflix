import { useState, useCallback } from 'react';

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api';
import { toast } from '@/lib/hooks/use-toast';
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
import PersonBox from '@/components/PersonBox';
import { Film } from '@/lib/types';
import { Button } from '@/components/ui/button';
import { Cross2Icon, PlusIcon } from '@radix-ui/react-icons';

function useDirectorCreate(film: Film | undefined) {
  const [open, setOpen] = useState<boolean>(false);
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

  const createDirectorMutation = useMutation({
    mutationFn: async (formValues: FormSchema) => {
      await invoke('create_director', {
        filmId: film?.id,
        personId: formValues?.person,
      });
    },
    onSuccess: () => {
      setOpen(false);
      toast({
        title: 'Director added',
        description: `Director was successfully added`,
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
        title: 'Failed to create the director',
        description: error.message,
      });
    },
  });

  const { reset } = form;

  const onSuccess = useCallback((values: FormSchema) => {
    createDirectorMutation.mutate(values);
  }, []);

  const directorCreate = useCallback(() => {
    reset({ person: undefined });
    setOpen(true);
  }, []);

  const CreateDialog = useCallback(() => {
    return (
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogOverlay>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Add Director</DialogTitle>
              <DialogDescription>Adding director</DialogDescription>
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
                    disabled={createDirectorMutation.isPending}
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

  return { directorCreate, CreateDialog };
}

export default useDirectorCreate;
