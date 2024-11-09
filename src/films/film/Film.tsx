import { useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { Film, Genre, Language } from '@/lib/types';
import { invoke } from '@tauri-apps/api/tauri';
import ReactPlayer from 'react-player';
import { forwardRef, memo, useMemo, useRef, useState } from 'react';
import screenfull from 'screenfull';
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
  ExternalLinkIcon,
  OpenInNewWindowIcon,
  PlayIcon,
  ResumeIcon,
  StarIcon,
} from '@radix-ui/react-icons';
import { Badge } from '@/components/ui/badge';
import { cn, isValidDate } from '@/lib/utils';
import { Separator } from '@/components/ui/separator';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';

const ICON_STYLE = 'h-5 w-5 mt-0.5 mr-1';

export default function FilmPage() {
  const { filmId } = useParams();

  const [isPlaying, setIsPlaying] = useState(false);
  const playerRef = useRef<null | ReactPlayer>(null);

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

  const resumeDisabled = useMemo(
    () => typeof film?.left_off_point !== 'number' || film.left_off_point <= 0,
    [film?.left_off_point]
  );

  function handlePlay() {
    setIsPlaying(true);
  }

  function handleResume() {
    if (resumeDisabled) return;
    playerRef?.current?.seekTo(film?.left_off_point ?? 0, 'seconds');
    handlePlay();
  }

  function handlePlayWith() {}

  function handleOpenIMDb() {}

  if (isFetchingFilm)
    return <div className='text-center p-4'>Getting film info...</div>;
  if (isFilmError)
    return (
      <div className='text-center p-4 text-red-500'>Error getting film</div>
    );

  return (
    <div className='min-h-screen min-w-screen border-2 border-red-500 p-4'>
      <div className='flex'>
        <div className='border-2 border-red-500 flex-[2]'>
          <ReactPlayer
            controls={true}
            url={`http://localhost:3000/film/${film?.id}`}
            playing={isPlaying}
            ref={playerRef}
            width='100%'
            height='100%'
          />
        </div>
        <div className='border-2 border-red-500 flex-1 p-2'>
          <Card>
            <CardHeader>
              <CardTitle className='text-xl'>{film?.title}</CardTitle>
              <CardDescription className='text-base'>
                {film?.plot}
              </CardDescription>
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
                  className='flex-[1.1] whitespace-nowrap'
                />
              </div>

              <FilmGenres genres={film?.genres} />

              <FilmLanguages languages={film?.languages} />

              <Separator className='mt-4' />

              <div className='flex mt-4 select-none gap-3'>
                <Button
                  variant='default'
                  className='flex-1'
                  onClick={handlePlay}
                >
                  <PlayIcon className={ICON_STYLE} /> Play
                </Button>
                <Button
                  variant='outline'
                  disabled={resumeDisabled}
                  className='flex-1'
                  onClick={handleResume}
                >
                  <ResumeIcon className={cn(ICON_STYLE, 'mr-1.5')} /> Resume
                </Button>
                <Button
                  variant='outline'
                  className='flex-1'
                  onClick={handlePlayWith}
                >
                  <OpenInNewWindowIcon className={ICON_STYLE} /> Play with
                </Button>
                <Button
                  variant='outline'
                  className='flex-1'
                  onClick={handleOpenIMDb}
                >
                  <ExternalLinkIcon className={ICON_STYLE} />
                  Open IMDb Page
                </Button>
              </div>

              <ScrollArea></ScrollArea>
            </CardContent>
          </Card>
        </div>
      </div>
      <div className='border-2 border-red-500'></div>
    </div>
  );
}

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
        <StarIcon className={cn(ICON_STYLE, colorClass)} />
        {rating}
      </div>
    );
  })
);

const FilmColor = memo(
  forwardRef<
    HTMLDivElement,
    { hasColor: boolean | undefined } & React.HTMLAttributes<HTMLDivElement>
  >(function FilmColor({ hasColor, className, ...props }, ref) {
    return (
      <div ref={ref} className={cn('flex', className)} {...props}>
        {hasColor ? (
          <CheckIcon className={cn(ICON_STYLE, 'text-green-300')} />
        ) : (
          <Cross2Icon className={cn(ICON_STYLE, 'text-red-300')} />
        )}
        Color
      </div>
    );
  })
);

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
        <ClockIcon className={ICON_STYLE} />
        {runtimeString}
      </div>
    );
  })
);

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
        <CalendarIcon className={ICON_STYLE} />
        {date.toLocaleDateString()}
      </div>
    );
  })
);

const FilmGenres = memo(
  forwardRef<
    HTMLDivElement,
    { genres: Genre[] | undefined } & React.HTMLAttributes<HTMLDivElement>
  >(function FilmGenres({ genres, ...props }, ref) {
    if (!Array.isArray(genres)) return '';

    const label = genres?.length > 1 ? 'Genres' : 'Genre';

    return (
      <div ref={ref} className={cn('pt-2', props.className)} {...props}>
        <div className='flex'>{label}</div>
        <div className='flex gap-1 pt-1'>
          {genres.map((genres) => (
            <Badge>{genres.name}</Badge>
          ))}
        </div>
      </div>
    );
  })
);

const FilmLanguages = memo(
  forwardRef<
    HTMLDivElement,
    { languages: Language[] | undefined } & React.HTMLAttributes<HTMLDivElement>
  >(function FilmLanguages({ languages, ...props }, ref) {
    if (!Array.isArray(languages)) return '';

    const label = languages?.length > 1 ? 'Languages' : 'Language';

    return (
      <div ref={ref} className={cn('pt-2', props.className)} {...props}>
        <div className='flex'>{label}</div>
        <div className='flex gap-1 pt-1'>
          {languages.map((languages) => (
            <Badge>{languages.name}</Badge>
          ))}
        </div>
      </div>
    );
  })
);
