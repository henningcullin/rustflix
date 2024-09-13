import { useParams } from 'react-router-dom';
import { format } from 'date-fns';
import { Button } from '@/components/ui/button';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import { Textarea } from '@/components/ui/textarea';
import { cn, getFilmName } from '@/lib/utils';
import { CalendarIcon, FileIcon, IdCardIcon } from '@radix-ui/react-icons';
import { Calendar } from '@/components/ui/calendar';
import SelectFilmPopup from './SelectFilmPopup';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { Film } from '@/lib/types';
import { useEffect } from 'react';
import { formSchema, FormSchema } from './formUtils';
import { Separator } from '@/components/ui/separator';
import DirectoryIcon from '@/components/icons/DirectoryIcon';
import ValueDisplay from './ValueDisplay';

function EditFilm() {
  const { filmId } = useParams();
  const queryClient = useQueryClient();

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      imdb_id: '',
      title: '',
      release_date: new Date(),
      plot: '',
      run_time: 0,
      has_color: false,
      rating: 0,
      has_watched: false,
      left_off_point: 0,
      registered: false,
      genres: [],
      directors: [],
      stars: [],
      languages: [],
      keywords: [],
    },
  });

  // Fetch film details using useQuery
  const {
    data: film,
    isLoading: isFetchingFilm,
    isError: isFilmError,
  } = useQuery<Film, Error>({
    queryKey: ['film', filmId],
    queryFn: async () => {
      if (!filmId) throw new Error('Invalid filmId');
      const id = parseInt(filmId);
      if (isNaN(id)) throw new Error('Invalid filmId');
      const result = await invoke<Film>('get_film', { id });
      if (!result) throw new Error('Film not found');
      return result;
    },
    enabled: !!filmId,
  });

  // Update form when film data is available
  useEffect(() => {}, [film, form]);

  // Mutation for scraping film
  const scrapeFilmMutation = useMutation<
    boolean,
    Error,
    { imdbId: string; databaseId: number }
  >({
    mutationFn: async ({ imdbId, databaseId }) => {
      return invoke<boolean>('scrape_film', { imdbId, databaseId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['film', filmId] });
      queryClient.invalidateQueries({ queryKey: ['films'] });
    },
    onError: (error) => {
      console.error('Scraping failed:', error);
    },
  });

  // Handling the film selection
  const handleFilmSelect = (imdbId: string | undefined) => {
    if (!filmId || !imdbId) return;
    const databaseId = parseInt(filmId);
    if (isNaN(databaseId)) return;

    scrapeFilmMutation.mutate({ imdbId, databaseId });
  };

  function onSuccess(values: FormSchema) {
    console.log(values);
  }

  function onError() {
    console.log('error');
  }

  if (isFetchingFilm) return <p>Loading film data...</p>;

  if (isFilmError) return <p>Failed to load film.</p>;

  return (
    <div>
      <div className='container mx-auto rounded-md border p-12'>
        <div className='pb-4 text-3xl font-bold'>
          Editing film
          {getFilmName(film?.file)?.length
            ? `: ${getFilmName(film?.file)}`
            : ''}
        </div>
        <ValueDisplay
          label='Id'
          value={film?.id?.toString()}
          icon={<IdCardIcon className='h-6 w-6' aria-label='Id Card Icon' />}
        />
        <ValueDisplay
          label='Directory'
          value={film?.directory?.path}
          icon={
            <DirectoryIcon className='h-6 w-6' aria-label='Directory Icon' />
          }
        />
        <ValueDisplay
          label='File'
          value={film?.file}
          icon={<FileIcon className='h-6 w-6' aria-label='File Icon' />}
        />
      </div>

      <Separator className='mt-8 mb-8' />

      <Form {...form}>
        <form
          onSubmit={form.handleSubmit(onSuccess, onError)}
          className='container mx-auto'
        >
          <FormField
            control={form.control}
            name='imdb_id'
            render={({ field }) => (
              <FormItem>
                <FormLabel>IMDB Id</FormLabel>
                <FormControl>
                  <div className='flex gap-3'>
                    <Input placeholder='' {...field} />
                    <SelectFilmPopup
                      onSelect={handleFilmSelect}
                      filePath={film?.file}
                    />
                  </div>
                </FormControl>
                <FormDescription>The id of the film on IMDB</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name='title'
            render={({ field }) => (
              <FormItem>
                <FormLabel>Title</FormLabel>
                <FormControl>
                  <Input placeholder='Enter the title' {...field} />
                </FormControl>
                <FormDescription>Title of the motion picture</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name='plot'
            render={({ field }) => (
              <FormItem>
                <FormLabel>Plot</FormLabel>
                <FormControl>
                  <Textarea placeholder='Enter the plot' {...field} />
                </FormControl>
                <FormDescription>Plot of the motion picture</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name='release_date'
            render={({ field }) => (
              <FormItem>
                <FormLabel>Release Date</FormLabel>
                <br />
                <Popover>
                  <PopoverTrigger asChild>
                    <FormControl>
                      <Button
                        variant={'outline'}
                        className={cn(
                          'w-[240px] pl-3 text-left font-normal',
                          !field.value && 'text-muted-foreground'
                        )}
                      >
                        {field.value ? (
                          format(field.value, 'PPP')
                        ) : (
                          <span>Pick a date</span>
                        )}
                        <CalendarIcon className='ml-auto h-4 w-4 opacity-50' />
                      </Button>
                    </FormControl>
                  </PopoverTrigger>
                  <PopoverContent className='w-auto p-0' align='start'>
                    <Calendar
                      mode='single'
                      selected={field.value}
                      onSelect={field.onChange}
                      disabled={(date: Date) =>
                        date > new Date() || date < new Date('1900-01-01')
                      }
                      initialFocus
                    />
                  </PopoverContent>
                </Popover>
                <FormDescription>
                  Release date of the motion picture
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <Button type='submit' disabled={scrapeFilmMutation.isPending}>
            {scrapeFilmMutation.isPending ? 'Scraping...' : 'Submit'}
          </Button>
        </form>
      </Form>
    </div>
  );
}

export default EditFilm;
