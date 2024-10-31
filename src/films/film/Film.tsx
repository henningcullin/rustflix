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

  const handlePlay = () => {
    setIsPlaying(true);
    setStartFromLeftOff(false);
    document.getElementById('player')?.requestFullscreen();
  };

  const handleResume = () => {
    setIsPlaying(true);
    setStartFromLeftOff(true);
    document.getElementById('player')?.requestFullscreen();
  };

  // Helper to determine color for rating
  function ratingColor(rating: number | undefined) {
    if (rating === undefined) return '';

    // Ensure the rating is within the expected range
    rating = Math.max(0, Math.min(10, rating));

    // Calculate the red and green components
    const red = Math.round(((10 - rating) * 255) / 10); // 0 at 10, 255 at 0
    const green = Math.round((rating * 255) / 10); // 0 at 0, 255 at 10

    // Create the color in hex format
    const color = `#${((1 << 24) + (red << 16) + (green << 8))
      .toString(16)
      .slice(1)}`;

    // Return the Tailwind CSS class
    return color;
  }

  return (
    <div className='grid grid-cols-1 lg:grid-cols-2 lg:grid-rows-2 h-screen'>
      {/* Top Left: Video Player */}
      <div className='h-1/2 lg:h-full flex flex-col items-center justify-center w-10/12'>
        <div
          id='player'
          className='relative rounded-lg overflow-hidden shadow-lg w-full h-full'
        >
          <ReactPlayer
            url={`http://localhost:3000/film/${film?.id}`}
            controls
            playing={isPlaying}
            startTime={
              startFromLeftOff && film?.left_off_point ? film.left_off_point : 0
            }
            width='100%'
            height='100%'
          />
        </div>
        <div className='flex space-x-4 mt-4'>
          <Button onClick={handlePlay}>
            <PlayIcon className='mr-1' />
            Play
          </Button>
          <Button onClick={handleResume} disabled={!film?.left_off_point}>
            <ResumeIcon className='mr-1' />
            Resume{' '}
            {film?.left_off_point
              ? `from ${Math.floor(film.left_off_point / 60)}:${
                  film.left_off_point % 60
                }`
              : ''}
          </Button>
        </div>
      </div>

      {/* Top Right: Metadata Section */}
      <div className='lg:h-full p-4 flex flex-col justify-between w-8/12'>
        <div>
          <h2 className='text-3xl font-semibold'>{film?.title}</h2>
          <p>{film?.plot}</p>
          <div className='flex flex-wrap items-center space-x-2 space-y-2 mt-2'>
            <span className='font-medium'>Genres:</span>
            {film?.genres.map((genre) => (
              <Badge key={genre.id}>{genre.name}</Badge>
            ))}
          </div>
          <div className='flex flex-wrap items-center space-x-2 space-y-2 mt-2'>
            <GlobeIcon className='text-blue-400 mt-2' />
            <span className='font-medium'>Languages:</span>
            {film?.languages.map((lang) => (
              <Badge key={lang.id}>{lang.name}</Badge>
            ))}
          </div>
          <div className='flex items-center space-x-2 mt-2'>
            <StarIcon className='text-yellow-400' />
            <span className='font-medium'>Rating:</span>
            <span
              className={clsx(`px-1 py-1 rounded-full font-semibold`)}
              style={{ color: `${ratingColor(film?.rating)}` }}
            >
              {film?.rating || 'N/A'}
            </span>
          </div>
          <p className='mt-2'>
            <strong>Runtime:</strong>{' '}
            {film?.run_time
              ? `${Math.floor(film.run_time / 3600)}h ${Math.floor(
                  (film.run_time % 3600) / 60
                )}min`
              : 'N/A'}
          </p>
          <p>
            <strong>Release Date:</strong> {film?.release_date}
          </p>
        </div>
      </div>

      {/* Bottom Section: Combined Carousels */}
      <div className='col-span-2 flex flex-col h-1/2 w-full'>
        {/* Directors Carousel */}
        <div className='flex flex-col w-full h-1/2 p-4'>
          <h3 className='text-2xl font-semibold'>Directors</h3>
          <div className='flex overflow-x-auto space-x-4 pb-4 mt-2'>
            {film?.directors.map((director) => (
              <div key={director.id} className='min-w-[100px] flex-shrink-0'>
                <Avatar id={director.id.toString()} />
                <p className='text-center'>{director.name}</p>
              </div>
            ))}
          </div>
        </div>

        {/* Characters Carousel */}
        <div className='flex flex-col w-full h-1/2 p-4'>
          <h3 className='text-2xl font-semibold'>Characters</h3>
          <div className='flex overflow-x-auto space-x-4 pb-4 mt-2'>
            {film?.stars.map((star) => (
              <div key={star.actor.id} className='min-w-[100px] flex-shrink-0'>
                <Avatar id={star.actor.id.toString()} />
                <p className='text-center'>{star.actor.name}</p>
                <p className='text-center text-sm'>{star.description}</p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default FilmPage;
