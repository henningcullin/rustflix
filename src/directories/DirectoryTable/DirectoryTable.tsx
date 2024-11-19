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
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { invoke } from '@tauri-apps/api/tauri';
import {
  DotsHorizontalIcon,
  ExclamationTriangleIcon,
  InfoCircledIcon,
  Pencil2Icon,
  TrashIcon,
  UpdateIcon,
} from '@radix-ui/react-icons';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Directory } from '@/lib/types';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { cn } from '@/lib/utils';
import { useToast } from '@/hooks/use-toast';
import { Button } from '@/components/ui/button';

const ALERT_STYLE = 'max-w-[600px] w-full mx-auto';
const ICON_STYLE = 'h-5 w-5';

export function DirectoryTable() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

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
      queryClient.invalidateQueries({ queryKey: ['directories'] });
      toast({
        title: 'Character deleted',
        description: `Directory was successfully removed`,
      });
    },
    onError: (error: Error) => {
      toast({
        variant: 'destructive',
        title: 'Failed to delete the directory',
        description: error.message,
      });
      console.error('Failed to delete directory', error);
    },
  });

  return isLoading ? (
    <Alert className={ALERT_STYLE}>
      <UpdateIcon className={cn(ICON_STYLE, 'animate-spin')} />
      <AlertTitle>One second</AlertTitle>
      <AlertDescription>Getting the directories for you...</AlertDescription>
    </Alert>
  ) : isError ? (
    <Alert variant='destructive' className={ALERT_STYLE}>
      <ExclamationTriangleIcon className={ICON_STYLE} />
      <AlertTitle>Error</AlertTitle>
      <AlertDescription>
        An error occured while getting the films
        {error.message}
      </AlertDescription>
    </Alert>
  ) : !directories ||
    typeof directories?.length !== 'number' ||
    !directories?.length ? (
    <Alert className={ALERT_STYLE}>
      <InfoCircledIcon className={ICON_STYLE} />
      <AlertTitle>Info</AlertTitle>
      <AlertDescription>No directories found</AlertDescription>
    </Alert>
  ) : (
    <>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Path</TableHead>
            <TableHead className='w-12'>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {directories.map((directory) => (
            <TableRow key={directory.id}>
              <TableCell>{directory.path}</TableCell>
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
                    <DropdownMenuGroup>
                      <DropdownMenuItem
                        onClick={() => {} /* directoryEdit(directory) */}
                      >
                        <Pencil2Icon className='w-5 h-5 mr-2' />
                        Edit
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => {} /* directoryDelete(directory) */}
                        disabled={deleteDirectoryMutation.isPending}
                      >
                        <TrashIcon className='w-5 h-5 mr-2' />
                        {deleteDirectoryMutation.isPending
                          ? 'Deleting...'
                          : 'Delete'}
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
