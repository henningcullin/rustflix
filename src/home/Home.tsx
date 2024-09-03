import { filmAtom } from '@/lib/atoms';
import { Film } from '@/lib/types';
import { invoke } from '@tauri-apps/api/tauri';
import { useAtom } from 'jotai';
import { useQuery } from 'react-query';

function Home() {
  const [films, setFilms] = useAtom(filmAtom);

  const { error, isLoading } = useQuery<Film[], Error>(
    'films',
    async () => {
      const data = await invoke<Film[]>('get_all_films');
      return data || [];
    },
    {
      onSuccess: (data) => {
        setFilms(data);
      },
    }
  );

  if (isLoading) {
    return <div>Loading...</div>;
  }

  if (error) {
    return <div>Error fetching films: {error.message}</div>;
  }

  return (
    <div>
      {films
        .filter((film) => film.registered)
        .map((film) => (
          <div key={film.id}>
            <h2>{film.title}</h2>
            <p>File: {film.file}</p>
          </div>
        ))}
    </div>
  );
}

export default Home;
