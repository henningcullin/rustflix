import { useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { Film } from '@/lib/types';
import { invoke } from '@tauri-apps/api/tauri';

function FilmPage() {
  const { filmId } = useParams();

  // Fetch film details using useQuery
  const {
    data: film,
    isLoading: isFetchingFilm,
    isError: isFilmError,
  } = useQuery<Film, Error>({
    queryKey: ['film', filmId],
    queryFn: async () => {
      if (!filmId) throw new Error('No film id specified');
      const id = parseInt(filmId);
      if (isNaN(id)) throw new Error('Invalid film id');
      const result = await invoke<Film>('get_film', { id });
      if (!result) throw new Error('Film not found');
      return result;
    },
    enabled: !!filmId,
  });

  if (isFetchingFilm) return <div>Getting film info...</div>;

  if (isFilmError) return <div>Error getting film</div>;

  return (
    <div>
      <h2>Should show it, {film?.file}</h2>
      <video src={`localhost:3000/film/${film?.id}`}></video>
    </div>
  );
}

export default FilmPage;
