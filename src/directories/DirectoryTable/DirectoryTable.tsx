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
  TrashIcon,
  UpdateIcon,
} from '@radix-ui/react-icons';
import { useQuery } from '@tanstack/react-query';
import { Directory } from '@/components/lib/types';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { cn } from '@/components/lib/utils';
import { Button } from '@/components/ui/button';
import useDirectoryRemove from './useDirectoryRemove';

const ALERT_STYLE = 'max-w-[600px] w-full mx-auto';
const ICON_STYLE = 'h-5 w-5';

export function DirectoryTable() {
  const { directoryRemove, RemoveDialog } = useDirectoryRemove();

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
      <RemoveDialog />
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Path</TableHead>
            <TableHead className='w-16'>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {directories.map((directory) => (
            <TableRow key={directory.id}>
              <TableCell>{directory.path}</TableCell>
              <TableCell className='inline-flex place-content-center w-16'>
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
                        onClick={() => directoryRemove(directory)}
                      >
                        <TrashIcon className='w-5 h-5 mr-2' />
                        Remove
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
