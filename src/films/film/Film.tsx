import { useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { Film } from '@/lib/types';
import { invoke } from '@tauri-apps/api/tauri';
import ReactPlayer from 'react-player';
import { forwardRef, memo, useState } from 'react';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  CalendarIcon,
  CheckIcon,
  ClockIcon,
  Cross2Icon,
  StarIcon,
} from '@radix-ui/react-icons';
import { Badge } from '@/components/ui/badge';
import { cn, isValidDate } from '@/lib/utils';

export default function FilmPage() {
  const { filmId } = useParams();

  const [isPlaying, setIsPlaying] = useState(false);

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
        <div className='border-2 border-red-500 flex-[2]'>
          <ReactPlayer
            url={`http://localhost:3000/film/${film?.id}`}
            controls
            playing={isPlaying}
            width='100%'
            height='100%'
          />
        </div>
        <div className='border-2 border-red-500 flex-1'>
          <Card>
            <CardHeader>
              <CardTitle>{film?.title}</CardTitle>
              <CardDescription>{film?.plot}</CardDescription>
            </CardHeader>
            <CardContent>
              <div className='flex flex-wrap gap-2 2xl:flex-row sm:flex-col w-full'>
                <FilmRating
                  rating={film?.rating}
                  className='flex-[0.5] whitespace-nowrap'
                />
                <FilmColor
                  hasColor={film?.has_color}
                  className='flex-[0.6] whitespace-nowrap'
                />
                <FilmRuntime
                  runtime={film?.run_time}
                  className='flex-[0.9] whitespace-nowrap'
                />
                <FilmReleaseDate
                  release_date={film?.release_date}
                  className='flex-[1.6] whitespace-nowrap'
                />
              </div>

              <div className='pt-2'>
                <p>Genres</p>
                <div className='flex gap-1 pt-1'>
                  {film?.genres.map((genre) => (
                    <Badge>{genre.name}</Badge>
                  ))}
                </div>
              </div>
              <div className='pt-2'>
                <p>Languages</p>
                <div className='flex gap-1 pt-1'>
                  {film?.languages.map((languages) => (
                    <Badge>{languages.name}</Badge>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
      <div className='border-2 border-red-500 h-72'>
        Directors and Characters
      </div>
    </div>
  );
}

// Component: FilmRating
const FilmRating = memo(
  forwardRef<
    HTMLDivElement,
    { rating: number | undefined } & React.HTMLAttributes<HTMLDivElement>
  >(function FilmRating({ rating, className, ...props }, ref) {
    if (typeof rating !== 'number') return null;

    let colorClass = '';
    if (rating >= 9.1) colorClass = 'text-green-300';
    else if (rating >= 8.4) colorClass = 'text-lime-300';
    else if (rating >= 6.8) colorClass = 'text-yellow-300';
    else if (rating >= 5.3) colorClass = 'text-orange-300';
    else colorClass = 'text-red-300';

    return (
      <div ref={ref} className={cn('flex', className)} {...props}>
        <StarIcon className={cn('h-5 w-5 mt-0.5 mr-1', colorClass)} />
        {rating}
      </div>
    );
  })
);

// Component: FilmColor
const FilmColor = memo(
  forwardRef<
    HTMLDivElement,
    { hasColor: boolean | undefined } & React.HTMLAttributes<HTMLDivElement>
  >(function FilmColor({ hasColor, className, ...props }, ref) {
    return (
      <div ref={ref} className={cn('flex', className)} {...props}>
        {hasColor ? (
          <CheckIcon className='h-5 w-5 mt-0.5 mr-1 text-green-300' />
        ) : (
          <Cross2Icon className='h-5 w-5 mt-0.5 mr-1 text-red-300' />
        )}
        Color
      </div>
    );
  })
);

// Component: FilmRuntime
const FilmRuntime = memo(
  forwardRef<
    HTMLDivElement,
    { runtime: number | undefined } & React.HTMLAttributes<HTMLDivElement>
  >(function FilmRuntime({ runtime, className, ...props }, ref) {
    if (typeof runtime !== 'number') return null;

    const hours = Math.floor(runtime / 60 / 60);
    const minutes = (runtime / 60) % 60;

    const runtimeBuilder = [];
    if (hours) runtimeBuilder.push(`${hours}h`);
    if (minutes) runtimeBuilder.push(`${minutes}m`);

    const runtimeString = runtimeBuilder.join(' ');
    if (!runtimeString?.length) return null;

    return (
      <div ref={ref} className={cn('flex', className)} {...props}>
        <ClockIcon className='h-5 w-5 mt-0.5 mr-1' />
        {runtimeString}
      </div>
    );
  })
);

// Component: FilmReleaseDate
const FilmReleaseDate = memo(
  forwardRef<
    HTMLDivElement,
    { release_date: string | undefined } & React.HTMLAttributes<HTMLDivElement>
  >(function FilmReleaseDate({ release_date, className, ...props }, ref) {
    if (typeof release_date !== 'string' || !isValidDate(release_date))
      return null;

    const date = new Date(release_date);

    return (
      <div ref={ref} className={cn('flex', className)} {...props}>
        <CalendarIcon className='h-5 w-5 mt-0.5 mr-1' />
        {date.toLocaleDateString()}
      </div>
    );
  })
);
