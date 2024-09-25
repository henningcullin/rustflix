import Avatar from '@/components/Avatar';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from '@/components/ui/hover-card';
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Film } from '@/lib/types';
import {
  DotsHorizontalIcon,
  Pencil2Icon,
  TrashIcon,
} from '@radix-ui/react-icons';
import useDirectorDelete from './useDirectorDelete';
import useDirectorEdit from './useDirectorEdit';

function DirectorTable({ film }: { film: Film | undefined }) {
  const { directorDelete, DeleteDialog } = useDirectorDelete(film);
  const { directorEdit, EditDialog } = useDirectorEdit(film);

  return (
    <>
      <EditDialog />
      <DeleteDialog />
      <Table>
        <TableCaption>List of all characters in the film</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead>Avatar</TableHead>
            <TableHead>Actor</TableHead>
            <TableHead className='w-12'>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {film?.directors?.map((director) => (
            <TableRow key={director.id}>
              <TableCell>
                <Avatar id={director.id}></Avatar>
              </TableCell>
              <TableCell>
                <HoverCard>
                  <HoverCardTrigger>
                    <span className='hover:underline underline-offset-4 cursor-pointer'>
                      {director.name}
                    </span>
                  </HoverCardTrigger>
                  <HoverCardContent>
                    <b>{director.id}</b>
                    <br />
                    <i>{director.imdb_id}</i>
                  </HoverCardContent>
                </HoverCard>
              </TableCell>
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
                      <DropdownMenuItem onClick={() => directorEdit(director)}>
                        <Pencil2Icon className='w-5 h-5 mr-2' />
                        Edit
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => directorDelete(director)}
                      >
                        <TrashIcon className='w-5 h-5 mr-2' />
                        Delete
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

export default DirectorTable;
