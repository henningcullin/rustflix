import { useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { Film } from '@/lib/types';
import { invoke } from '@tauri-apps/api/tauri';
import ReactPlayer from 'react-player';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Avatar } from '@/components/ui/avatar';
import clsx from 'clsx';
import {
  GlobeIcon,
  PlayIcon,
  ResumeIcon,
  StarIcon,
} from '@radix-ui/react-icons';

function FilmPage() {
  const { filmId } = useParams();
  const [isPlaying, setIsPlaying] = useState(false);
  const [startFromLeftOff, setStartFromLeftOff] = useState(false);

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

  if (isFetchingFilm)
    return <div className='text-center p-4'>Getting film info...</div>;
  if (isFilmError)
    return (
      <div className='text-center p-4 text-red-500'>Error getting film</div>
    );

  return (
    <div className='min-h-screen min-w-screen border-2 border-red-500 p-4'>
      <div className='flex'>
        <div className='border-2 border-red-500 flex-1'>Poster area</div>
        <div className='border-2 border-red-500 flex-1'>Player area</div>
        <div className='border-2 border-red-500 flex-1'>Meta info area</div>
      </div>
      <div className='border-2 border-red-500 h-72'>
        Directors and Characters
      </div>
    </div>
  );
}

export default FilmPage;
