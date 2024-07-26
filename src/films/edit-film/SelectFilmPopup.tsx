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

interface SearchItem {
  i: { height: number; imageUrl: string; width: number };
  id: string;
  l: string;
  q: string;
  qid: string;
  rank: number;
  s: string;
  y: number;
}

async function searchFilms(
  searchValue: string
): Promise<SearchItem[] | boolean> {
  try {
    const url = `https://v3.sg.media-imdb.com/suggestion/x/${searchValue}.json?includeVideos=0`;
    const response: string = await invoke('fetch_data', { url });
    const parsed: {
      d: SearchItem[];
    } = JSON.parse(response);
    const items = parsed.d;
    const filmItems = items.filter((item) => item.id.includes('tt'));
    return filmItems;
  } catch (error) {
    console.error('Error searching for films', error);
    return false;
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
  const [searchItems, setSearchItems] = useState<SearchItem[]>([]);

  const filmName = getFilmName(filePath);

  const handleSearch = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.target as HTMLFormElement);
    const searchValue = formData.get('searchValue');
    if (typeof searchValue !== 'string') return;
    const items = await searchFilms(searchValue);
    if (typeof items === 'boolean') return;
    setSearchItems(items);
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
          <Input name='searchValue' defaultValue={filmName} />
          <Button>Search</Button>
        </form>
        <FilmList films={searchItems}></FilmList>
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

function FilmList({ films }: { films: SearchItem[] }) {
  return (
    <ul>
      {films.map((film) => (
        <li>{film.id}</li>
      ))}
    </ul>
  );
}

export default SelectFilmPopup;
