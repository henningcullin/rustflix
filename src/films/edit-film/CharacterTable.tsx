import Avatar from '@/components/Avatar';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
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
import { Character, Film } from '@/lib/types';
import {
  DotsHorizontalIcon,
  EyeOpenIcon,
  Pencil2Icon,
  TrashIcon,
} from '@radix-ui/react-icons';
import { useCallback, useState } from 'react';

function CharacterTable({ film }: { film: Film | undefined }) {
  const [isAlertOpen, setIsAlertOpen] = useState<boolean>(false); // Controls alert visibility
  const [selectedCharacter, setSelectedCharacter] = useState<Character | null>(
    null
  ); // Tracks selected character

  // Function to handle the delete callback
  const handleDelete = useCallback(() => {
    if (selectedCharacter) {
      console.log('Delete confirmed for character:', selectedCharacter);
      // Here you can call your delete logic or API to delete the character
      setIsAlertOpen(false); // Close the dialog after action
    }
  }, [selectedCharacter]);

  const openDeleteConfirm = (character: Character) => {
    setSelectedCharacter(character); // Set selected character
    setIsAlertOpen(true); // Open the dialog
  };

  return (
    <>
      <AlertDialog open={isAlertOpen} onOpenChange={setIsAlertOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Confirm Deletion</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete {selectedCharacter?.description}?
              This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDelete}>Delete</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
      <Table>
        <TableCaption>List of all characters in the film</TableCaption>
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
            <TableRow>
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
              <TableCell className=''>
                <DropdownMenu>
                  <DropdownMenuTrigger>
                    <Button variant='outline' className='w-10 h-10 p-0'>
                      <DotsHorizontalIcon />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent>
                    <DropdownMenuLabel>Actions</DropdownMenuLabel>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem>
                      <EyeOpenIcon className='w-5 h-5 mr-2' />
                      View
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuGroup>
                      <DropdownMenuItem>
                        <Pencil2Icon className='w-5 h-5 mr-2' />
                        Edit
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => openDeleteConfirm(character)}
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
