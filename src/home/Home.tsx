import Cover from '@/components/Cover';
import { Film } from '@/lib/types';
import { invoke } from '@tauri-apps/api/tauri';
import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from '@/components/ui/accordion';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import {
  ExclamationTriangleIcon,
  EyeNoneIcon,
  EyeOpenIcon,
  UpdateIcon,
} from '@radix-ui/react-icons';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';

function Home() {
  const {
    data: films,
    isError,
    isLoading,
  } = useQuery<Film[], Error>({
    queryKey: ['films'],
    queryFn: async () => {
      const data = await invoke<Film[]>('get_all_films');
      return data || [];
    },
  });

  return (
    <div>
      {isLoading ? (
        <div className='px-5 w-full'>
          <Alert className='w-full'>
            <UpdateIcon className='h-4 w-4 animate-spin' />
            <AlertTitle>One second</AlertTitle>
            <AlertDescription>Retriving the films for you...</AlertDescription>
          </Alert>
        </div>
      ) : isError ? (
        <div className='px-5 w-full'>
          <Alert variant='destructive'>
            <ExclamationTriangleIcon className='h-4 w-4' />
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>
              An error occured while retrieving the films
            </AlertDescription>
          </Alert>
        </div>
      ) : !films || films?.length <= 0 ? (
        <div className='px-5 w-full'>
          <Alert variant='warning' className='px-5'>
            <ExclamationTriangleIcon className='h-4 w-4' />
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>No films found</AlertDescription>
          </Alert>
        </div>
      ) : (
        <>
          <Accordion type='single' collapsible className='px-5 w-full'>
            <AccordionItem value='filters'>
              <AccordionTrigger>Filters</AccordionTrigger>
              <AccordionContent>
                <ToggleGroup type='multiple'>
                  <ToggleGroupItem
                    value='watched'
                    aria-label='Toggle show watched films'
                  >
                    <EyeOpenIcon />
                  </ToggleGroupItem>
                  <ToggleGroupItem
                    value='unwatched'
                    aria-label='Toggle show unwatched films'
                  >
                    <EyeNoneIcon />
                  </ToggleGroupItem>
                </ToggleGroup>
              </AccordionContent>
            </AccordionItem>
          </Accordion>
          <div className='grid gap-6 p-6 gird-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5'>
            {films
              .filter((film) => film.registered)
              .slice(0, 15)
              .map((film) => (
                <FilmItem key={film.id} film={film} />
              ))}
          </div>
        </>
      )}
    </div>
  );
}

function FilmItem({ film }: { film: Film }) {
  return (
    <Link
      to={`/film/${film.id}`}
      className='aspect-[187,5/262,5] cursor-pointer transform transition-transform duration-300 hover:scale-105'
    >
      <Cover id={film.id}></Cover>
      <h1 className='mt-2 text-center text-lg font-medium truncate'>
        {film.title}
      </h1>
    </Link>
  );
}

export default Home;
