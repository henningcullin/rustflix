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
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Film } from '@/lib/types';
import {
  DotsHorizontalIcon,
  Pencil2Icon,
  PlusIcon,
  TrashIcon,
} from '@radix-ui/react-icons';
import useCharacterDelete from './useCharacterDelete';
import useCharacterEdit from './UseCharacterEdit';
import useCharacterCreate from './useCharacterCreate';

function CharacterTable({ film }: { film: Film | undefined }) {
  const { characterDelete, DeleteDialog } = useCharacterDelete(film);
  const { characterEdit, EditDialog } = useCharacterEdit(film);
  const { characterCreate, CreateDialog } = useCharacterCreate(film);

  return (
    <>
      <DeleteDialog />
      <EditDialog />
      <CreateDialog />
      <Button variant='outline' onClick={characterCreate} className='my-4'>
        <PlusIcon />
        Add character
      </Button>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Avatar</TableHead>
            <TableHead>Description</TableHead>
            <TableHead>Actor</TableHead>
            <TableHead className='w-12'>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {film?.stars?.map((character) => (
            <TableRow key={character.actor.id}>
              <TableCell>
                <Avatar id={character.actor.id}></Avatar>
              </TableCell>
              <TableCell>{character.description}</TableCell>
              <TableCell>
                <HoverCard>
                  <HoverCardTrigger>
                    <span className='hover:underline underline-offset-4 cursor-pointer'>
                      {character.actor.name}
                    </span>
                  </HoverCardTrigger>
                  <HoverCardContent>
                    <b>{character.actor.id}</b>
                    <br />
                    <i>{character.actor.imdb_id}</i>
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
                      <DropdownMenuItem
                        onClick={() => characterEdit(character)}
                      >
                        <Pencil2Icon className='w-5 h-5 mr-2' />
                        Edit
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => characterDelete(character)}
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

export default CharacterTable;
