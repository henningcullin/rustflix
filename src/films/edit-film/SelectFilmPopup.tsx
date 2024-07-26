import React, { useRef, useState } from 'react';

import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogClose,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Link1Icon, OpenInNewWindowIcon } from '@radix-ui/react-icons';
import { invoke } from '@tauri-apps/api/tauri';

interface Arguments {
  onSelect: (url: string | undefined) => void;
  filePath: string | undefined;
}

async function searchFilms(searchValue: string): Promise<string> {
  try {
    const data: string = await invoke('search-films', { searchValue });
    return data;
  } catch (error) {
    console.error('Error searching for films', error);
    return '';
  }
}

function getFilmName(filePath: string | undefined): string {
  const fileName = filePath?.split('\\')?.pop();
  const value =
    fileName
      ?.replace(/\.\d{4}.*\.(mp4|mkv)$/, '')
      // @ts-ignore
      ?.replaceAll('.', ' ') ?? '';
  return value;
}

function SelectFilmPopup({ onSelect, filePath }: Arguments) {
  const [open, setOpen] = useState<boolean>(false);

  const filmName = getFilmName(filePath);

  const handleSelect = () => {
    onSelect('');
    setOpen(false);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>Select Film</Button>
        <Link1Icon />
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Select film</DialogTitle>
        </DialogHeader>
        <DialogFooter>
          <DialogClose asChild>
            <Button type='button' variant='secondary'>
              Close
            </Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function FilmList() {
  return (
    <ul>
      <li></li>
    </ul>
  );
}

export default SelectFilmPopup;
