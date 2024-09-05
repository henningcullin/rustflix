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
import { z } from 'zod';
import { useForm } from 'react-hook-form';
import { Textarea } from '@/components/ui/textarea';
import { cn } from '@/lib/utils';
import { CalendarIcon } from '@radix-ui/react-icons';
import { Calendar } from '@/components/ui/calendar';
import SelectFilmPopup from './SelectFilmPopup';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { Film } from '@/lib/types';
import { useEffect } from 'react';

const formSchema = z.object({
  link: z.string(),
  title: z.string(),
  synopsis: z.string(),
  release_date: z.date(),
  duration: z.number(),
  cover_image: z.string(),
});

type FormSchema = z.infer<typeof formSchema>;

function EditFilm() {
  const { filmId } = useParams();
  const queryClient = useQueryClient();

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      link: '',
      title: '',
      synopsis: '',
      release_date: new Date(),
      duration: 0,
      cover_image: '',
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
  useEffect(() => {
    /* if (film) {
      form.reset({
        link: film.link,
        title: film.title,
        synopsis: film.synopsis,
        release_date: new Date(film.release_date),
        duration: film.duration,
        cover_image: film.cover_image,
      });
    } */
  }, [film, form]);

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
      <SelectFilmPopup onSelect={handleFilmSelect} filePath={film?.file} />

      <Form {...form}>
        <form
          onSubmit={form.handleSubmit(onSuccess, onError)}
          className='max-w-96 space-y-6 p-5'
        >
          <FormField
            control={form.control}
            name='link'
            render={({ field }) => (
              <FormItem>
                <FormLabel>Link</FormLabel>
                <FormControl>
                  <Input placeholder='Enter the link' {...field} />
                </FormControl>
                <FormDescription>Link to imdb source</FormDescription>
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
            name='synopsis'
            render={({ field }) => (
              <FormItem>
                <FormLabel>Synopsis</FormLabel>
                <FormControl>
                  <Textarea placeholder='Enter the synopsis' {...field} />
                </FormControl>
                <FormDescription>
                  Synopsis of the motion picture
                </FormDescription>
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
