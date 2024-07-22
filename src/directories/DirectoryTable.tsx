import { useEffect } from 'react';
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
import { Directory } from './Directories';
import { useAtom } from 'jotai';
import { directoryAtom } from '@/lib/atoms';
import { DotsVerticalIcon } from '@radix-ui/react-icons';

export function DirectoryTable() {
  const [directories, setDirectories] = useAtom(directoryAtom);

  async function getDirectories() {
    try {
      const data: Directory[] | null = await invoke('get_all_directories');
      if (data) setDirectories(data);
    } catch (error) {
      console.error(error);
    }
  }

  async function deleteDirectory(id: number) {
    try {
      const wasDeleted: boolean = await invoke('delete_directory', { id });
      if (wasDeleted) setDirectories(directories.filter((d) => d.id !== id));
    } catch (error) {
      console.error('Failed to delete directory', error);
    }
  }

  useEffect(() => {
    getDirectories();
  }, []);

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className='text-center'>Path</TableHead>
          <TableHead className='text-right max-w-2'>Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {directories.map((directory) => (
          <TableRow>
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
                    onClick={() => deleteDirectory(directory.id)}
                  >
                    Delete
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
