import React, { FormEventHandler, useRef, useState } from 'react';

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
import { Input } from '@/components/ui/input';
import { FormSubmitHandler } from 'react-hook-form';

interface Arguments {
  onSelect: (url: string | undefined) => void;
  filePath: string | undefined;
}

async function searchFilms(searchValue: string): Promise<string> {
  try {
    const url = `https://v3.sg.media-imdb.com/suggestion/x/${searchValue}.json?includeVideos=0`;
    const response: string = await invoke('fetch_data', { url });
    const parsed = JSON.parse(response);
    console.log(parsed);
    return '';
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
  const [searchItems, setSearchItems] = useState([]);

  const filmName = getFilmName(filePath);

  const handleSearch = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.target as HTMLFormElement);
    const searchValue = formData.get('searchValue');
    if (typeof searchValue === 'string') searchFilms(searchValue);
  };

  const handleSelect = () => {};

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>
          Select Film
          <Link1Icon />
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Select film</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSearch}>
          <Input name='searchValue' value={filmName} />
          <Button>Search</Button>
        </form>
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

function FilmList(films: string[]) {
  return (
    <ul>
      {films.map((film) => (
        <li>{film}</li>
      ))}
    </ul>
  );
}

export default SelectFilmPopup;
