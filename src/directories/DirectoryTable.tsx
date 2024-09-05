import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { invoke } from '@tauri-apps/api/tauri';
import { DotsVerticalIcon } from '@radix-ui/react-icons';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Skeleton } from '@/components/ui/skeleton';
import { Directory } from '@/lib/types';

export function DirectoryTable() {
  const queryClient = useQueryClient();

  // Fetch directories using useQuery
  const {
    data: directories,
    isLoading,
    isError,
    error,
  } = useQuery<Directory[], Error>({
    queryFn: async () => {
      const data = await invoke<Directory[]>('get_all_directories');
      return data || [];
    },
    queryKey: ['directories'],
  });

  // Mutation to delete a directory
  const deleteDirectoryMutation = useMutation({
    mutationFn: async (id: number) => {
      const wasDeleted = await invoke<boolean>('delete_directory', { id });
      return wasDeleted;
    },
    onSuccess: () => {
      // Invalidate and refetch the directories query after successful deletion
      queryClient.invalidateQueries({ queryKey: ['directories'] });
    },
    onError: (error: Error) => {
      console.error('Failed to delete directory', error);
    },
  });

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className='text-center'>Path</TableHead>
          <TableHead className='text-right max-w-2'>Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {isLoading ? (
          <TableRow>
            <TableCell>
              <Skeleton />
            </TableCell>
            <TableCell>
              <Skeleton />
            </TableCell>
          </TableRow>
        ) : isError ? (
          <TableRow>
            <TableCell>{error.message}</TableCell>
          </TableRow>
        ) : directories?.length ? (
          directories.map((directory) => (
            <TableRow key={directory.id}>
              <TableCell>{directory.path}</TableCell>
              <TableCell>
                <DropdownMenu>
                  <DropdownMenuTrigger>
                    <DotsVerticalIcon />
                  </DropdownMenuTrigger>
                  <DropdownMenuContent>
                    <DropdownMenuLabel>Actions</DropdownMenuLabel>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem
                      onClick={() =>
                        deleteDirectoryMutation.mutate(directory.id)
                      }
                      disabled={deleteDirectoryMutation.isPending}
                    >
                      {deleteDirectoryMutation.isPending
                        ? 'Deleting...'
                        : 'Delete'}
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </TableCell>
            </TableRow>
          ))
        ) : (
          <TableRow>
            <TableCell>No directories found</TableCell>
          </TableRow>
        )}
      </TableBody>
    </Table>
  );
}
