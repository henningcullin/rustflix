import { invoke } from '@tauri-apps/api/tauri';
import { ReactElement, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTrigger,
} from '@/components/ui/dialog';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Directory } from '@/lib/types';

export function DirectoryDialog({ children }: { children: ReactElement }) {
  const [directory, setDirectory] = useState<string>('');
  const [open, setOpen] = useState(false); // For controlling the dialog state
  const queryClient = useQueryClient();

  // Mutation to add a directory
  const addDirectoryMutation = useMutation<Directory, Error, string>({
    mutationFn: async (path: string) => {
      const dir = await invoke<Directory>('add_directory', { path });
      return dir;
    },
    onSuccess: () => {
      // Invalidate the 'directories' query to trigger refetching in the table
      queryClient.invalidateQueries({ queryKey: ['directories'] });
      // Clear the directory input and close the dialog
      setDirectory('');
      setOpen(false);
    },
    onError: (error: Error) => {
      console.error('Failed to add directory', error);
    },
  });

  // Function to select a directory
  async function selectDirectory() {
    try {
      const path: string | null = await invoke('select_directory');
      if (typeof path === 'string' && path) setDirectory(path);
    } catch (error) {
      console.error('Failed to select directory', error);
    }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger>{children}</DialogTrigger>
      <DialogContent>
        <DialogHeader></DialogHeader>
        <div className='w-[450px] p-1'>
          <Label>Directory</Label>
          <div className='flex gap-4'>
            <Input value={directory} readOnly disabled />
            <Button onClick={selectDirectory}>Select Directory</Button>
          </div>
          <br />
          <Button
            onClick={() => addDirectoryMutation.mutate(directory)}
            disabled={!directory || addDirectoryMutation.isPending}
          >
            {addDirectoryMutation.isPending ? 'Adding...' : 'Add Directory'}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
