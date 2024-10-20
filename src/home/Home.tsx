import Cover from '@/components/Cover';
import { Film } from '@/lib/types';
import { invoke } from '@tauri-apps/api/tauri';
import { useQuery } from '@tanstack/react-query';
import { Skeleton } from '@/components/ui/skeleton';
import { Link } from 'react-router-dom';

function Home() {
  const {
    data: films,
    error,
    isError,
    isLoading,
  } = useQuery<Film[], Error>({
    queryKey: ['films'],
    queryFn: async () => {
      const data = await invoke<Film[]>('get_all_films');
      return data || [];
    },
  });

  if (isError) {
    return <div>Error fetching films: {error.message}</div>;
  }

  return (
    <div className='grid gap-6 p-6 gird-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5'>
      {isLoading ? (
        Array.from({ length: 5 }).map((_, index) => (
          <Skeleton
            key={index}
            className='aspect-[187,5/262,5] w-full rounded-lg'
          />
        ))
      ) : !films ? (
        <b>No films found</b>
      ) : (
        films
          .filter((film) => film.registered)
          .map((film) => <FilmItem key={film.id} film={film} />)
      )}
    </div>
  );
}

function FilmItem({ film }: { film: Film }) {
  return (
    <Link
      to={`/film/${film.id}`}
      key={film.id}
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
