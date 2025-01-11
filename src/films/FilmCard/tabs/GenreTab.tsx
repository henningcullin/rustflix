import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { WithTooltip } from '@/components/WithTooltip';
import { Film, Genre } from '@/lib/types';
import { TrashIcon } from '@radix-ui/react-icons';
import React from 'react';

type GenreTabProps = {
  film: Film;
};

type ExtendedGenre = Genre & {
  selected: boolean;
};

export default function GenreTab({ film }: GenreTabProps) {
  const [genres, setGenres] = React.useState<ExtendedGenre[]>(
    film.genres.map((genre) => ({ ...genre, selected: false }))
  );

  function setSelected(id: number, selected: boolean) {
    const newGenres = genres.map<ExtendedGenre>((genre) =>
      genre.id === id ? { ...genre, selected } : genre
    );

    setGenres(newGenres);
  }

  return (
    <div className='w-full border-ws rounded-sm'>
      <div className='flex place-content-center w-full mb-4'>
        <div className='inline-flex gap-3'>
          <Button>New</Button>
        </div>
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>
              <Checkbox />
            </TableHead>
            <TableHead>ID</TableHead>
            <TableHead>Name</TableHead>
            <TableHead>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {genres.map((genre) => (
            <TableRow>
              <TableCell>
                <Checkbox
                  checked={genre.selected}
                  onCheckedChange={(checked) =>
                    setSelected(genre.id, !!checked)
                  }
                />
              </TableCell>
              <TableCell>{genre.id}</TableCell>
              <TableCell>{genre.name}</TableCell>
              <TableCell>
                <WithTooltip message='Remove genre from film'>
                  <Button variant='destructive' className='p-2'>
                    <TrashIcon className='w-5 h-5' />
                  </Button>
                </WithTooltip>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
