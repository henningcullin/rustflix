import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { OpenInNewWindowIcon } from '@radix-ui/react-icons';
import { SearchItem } from './SelectFilmPopup';

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

export default FilmList;
