import React, { useCallback, useEffect, useState } from 'react';

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
import {
  Cross2Icon,
  Link1Icon,
  MagnifyingGlassIcon,
  OpenInNewWindowIcon,
} from '@radix-ui/react-icons';
import { invoke } from '@tauri-apps/api/tauri';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { shell } from '@tauri-apps/api';
import { useQuery } from '@tanstack/react-query';

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

/**
 * Get films from imdb
 */
async function searchFilms(searchValue: string): Promise<SearchItem[]> {
  const url = `https://v3.sg.media-imdb.com/suggestion/x/${searchValue}.json?includeVideos=0`;
  const response: string = await invoke('fetch_data', { url });
  const parsed: { d: SearchItem[] } = JSON.parse(response);
  const items = parsed.d;
  const filmItems = items.filter((item) => item.id.includes('tt'));
  return filmItems;
}
/**
 * Sanitze filenames in attempt to get a search term for the film
 */
function getFilmName(filePath: string | undefined): string {
  const fileName = filePath?.split('\\')?.pop();
  const value = // @ts-ignore
    fileName?.replace(/\.\d{4}.*\.(mp4|mkv)$/, '')?.replaceAll('.', ' ') ?? '';
  return value;
}

function SelectFilmPopup({ onSelect, filePath }: Arguments) {
  const [open, setOpen] = useState<boolean>(false);
  const [searchTerm, setSearchTerm] = useState<string>(
    getFilmName(filePath) || ''
  );

  const {
    data: searchItems,
    isLoading,
    isError,
    error,
  } = useQuery<SearchItem[], Error>({
    queryKey: ['searchFilms', searchTerm],
    queryFn: () => searchFilms(searchTerm),
    enabled: !!searchTerm, // Only run query if searchTerm is not empty
  });

  function onSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const formData = new FormData(event.target as HTMLFormElement);
    const searchValue = formData.get('searchValue') as string;
    if (searchValue) {
      setSearchTerm(searchValue);
    }
  }

  useEffect(() => {
    setSearchTerm(getFilmName(filePath));
  }, [filePath]);

  const handleSelect = useCallback(
    (id: string) => {
      onSelect(id);
      setOpen(false);
    },
    [onSelect]
  );

  const handleLink = useCallback(
    (id: string) =>
      shell
        .open(`https://www.imdb.com/title/${id}/`)
        .catch((error) => console.error('Failed to open url', error)),
    []
  );

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>
          Select Film
          <Link1Icon className='ml-1' />
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Select film</DialogTitle>
        </DialogHeader>
        <form onSubmit={onSubmit} className='flex'>
          <Input
            name='searchValue'
            placeholder='Enter a film title'
            type={'search'}
            defaultValue={searchTerm}
          />
          <Button variant='secondary' className='ml-1'>
            Search
            <MagnifyingGlassIcon className='ml-1' />
          </Button>
        </form>
        {isLoading ? (
          <b>Loading...</b>
        ) : isError ? (
          <b>{error?.message}</b>
        ) : searchItems?.length ? (
          <FilmList
            films={searchItems}
            handleSelect={handleSelect}
            handleLink={handleLink}
          ></FilmList>
        ) : (
          <b>No films were found</b>
        )}
        <DialogFooter>
          <DialogClose asChild>
            <Button type='button' variant='secondary'>
              Close
              <Cross2Icon className='ml-1' />
            </Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function FilmList({
  films,
  handleSelect,
  handleLink,
}: {
  films: SearchItem[];
  handleSelect: (id: string) => void;
  handleLink: (id: string) => void;
}) {
  return (
    <ScrollArea className='max-h-[65dvh]'>
      <ul>
        {films.map((film) => (
          <>
            <li className='flex  min-h-76'>
              <div className='flex-1'>
                {film?.i?.imageUrl ? (
                  <img src={film.i.imageUrl} className='w-48' />
                ) : (
                  <div className='w-48 h-28'></div>
                )}
              </div>

              <div className='flex-1 grid h-full items-center '>
                <h4>
                  {film?.l} {film?.y ? `(${film.y})` : ''}
                </h4>
                <br />
                <Button
                  className='bg-green-600 hover:bg-green-700'
                  onClick={() => handleSelect(film.id)}
                >
                  Select
                </Button>
                <br />
                <Button
                  className='bg-sky-600 hover:bg-sky-700'
                  onClick={() => handleLink(film.id)}
                >
                  Open IMDB Page
                  <OpenInNewWindowIcon className='ml-1' />
                </Button>
              </div>
            </li>
            <Separator className='mt-4 mb-4' />
          </>
        ))}
      </ul>
    </ScrollArea>
  );
}

export default SelectFilmPopup;
