import Cover from '@/components/Cover';
import { Film } from '@/lib/types';
import { invoke } from '@tauri-apps/api/tauri';
import { useQuery } from '@tanstack/react-query';
import { Skeleton } from '@/components/ui/skeleton';
import { Link } from 'react-router-dom';

function Home() {
  // TODO:
  // - PersonBox async fetching of persons with limit and where query clauses
  // - Continue working on Editfilm component and make everything work
  // - ViewFilm/Film component for user friendly film details
  // - optimize rendering
  // - make image commands downscale images

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
    <div className='grid grid-cols-4 p-4'>
      {isLoading ? (
        <Skeleton className='w-[375px] h-[525px]'></Skeleton>
      ) : !films ? (
        <b>No films found</b>
      ) : (
        films.filter((film) => film.registered).map(FilmItem)
      )}
    </div>
  );
}

function FilmItem(film: Film) {
  return (
    <Link
      to={`/film/${film.id}`}
      key={film.id}
      className='w-[375px] h-[525px] cursor-pointer'
    >
      <h1 className=''>{film.title}</h1>
      <Cover id={film.id}></Cover>
    </Link>
  );
}

export default Home;
