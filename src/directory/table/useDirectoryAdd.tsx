import { useState, useCallback } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api';
import { toast } from '@/lib/hooks/use-toast';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogOverlay,
  DialogTitle,
} from '@/components/ui/dialog';
import { z } from 'zod';
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
import { Button } from '@/components/ui/button';
import { Cross2Icon, PlusIcon } from '@radix-ui/react-icons';

function useDirectoryAdd() {
  const [open, setOpen] = useState<boolean>(false);
  const queryClient = useQueryClient();

  const formSchema = z.object({
    path: z.string().min(1, 'A directory requires a path'),
  });

  type FormSchema = z.infer<typeof formSchema>;

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      path: '',
    },
  });

  const addDirectoryMutation = useMutation({
    mutationFn: async (formValues: FormSchema) => {
      await invoke('create_directory', {
        path: formValues.path,
      });
    },
    onSuccess: () => {
      setOpen(false);
      toast({
        title: 'Directory added',
        description: `Directory was successfully added`,
      });
      queryClient.invalidateQueries({
        queryKey: ['directories'],
      });
    },
    onError: (error) => {
      console.error(error);
      toast({
        variant: 'destructive',
        title: 'Failed to add the directory',
        description: error.message,
      });
    },
  });

  const { reset } = form;

  const onSuccess = useCallback((values: FormSchema) => {
    addDirectoryMutation.mutate(values);
  }, []);

  async function selectDirectory() {
    try {
      const path: string | null = await invoke('select_directory');
      if (typeof path === 'string' && path) form.setValue('path', path);
    } catch (error) {
      console.error('Failed to select directory', error);
      toast({
        variant: 'destructive',
        title: 'Failed to add the directory',
        description: 'An error occured while selecting directory',
      });
    }
  }

  const directoryAdd = useCallback(() => {
    reset({
      path: '',
    });
    setOpen(true);
  }, []);

  const AddDialog = useCallback(() => {
    return (
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogOverlay>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Add directory</DialogTitle>
            </DialogHeader>
            <Form {...form}>
              <form onSubmit={form.handleSubmit(onSuccess)}>
                <FormField
                  control={form.control}
                  name='path'
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Path</FormLabel>
                      <FormControl>
                        <div className='inline-flex gap-3 w-full'>
                          <Input {...field} />
                          <Button
                            type='button'
                            variant='outline'
                            onClick={selectDirectory}
                          >
                            Select Directory
                          </Button>
                        </div>
                      </FormControl>
                      <FormDescription>
                        The description of the character
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <DialogFooter>
                  <DialogClose>
                    <Button variant='outline'>
                      <Cross2Icon className='w-5 h-5 mr-2' />
                      Cancel
                    </Button>
                  </DialogClose>
                  <Button
                    type='submit'
                    variant='success'
                    disabled={addDirectoryMutation.isPending}
                  >
                    <PlusIcon className='w-5 h-5 mr-2' />
                    {addDirectoryMutation.isPending ? 'Adding...' : 'Add'}
                  </Button>
                </DialogFooter>
              </form>
            </Form>
          </DialogContent>
        </DialogOverlay>
      </Dialog>
    );
  }, [open]);

  return { directoryAdd, AddDialog };
}

export default useDirectoryAdd;
