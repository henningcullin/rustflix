import { useForm } from 'react-hook-form';
import { formSchema, FormSchema } from './formUtils';
import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { Film } from '@/components/lib/types';
import { useEffect } from 'react';
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
import SelectFilmPopup from './SelectFilmPopup';
import { Textarea } from '@/components/ui/textarea';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { Button } from '@/components/ui/button';
import { cn } from '@/components/lib/utils';
import { format } from 'date-fns';
import { CalendarIcon } from '@radix-ui/react-icons';
import { Calendar } from '@/components/ui/calendar';
import { Checkbox } from '@/components/ui/checkbox';

function MainForm({ film }: { film: Film | undefined }) {
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
    },
  });

  const { reset } = form;

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
      queryClient.invalidateQueries({
        queryKey: ['film', film?.id?.toString()],
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
      queryClient.invalidateQueries({ queryKey: ['persons'] });
    },
    onError: (error) => {
      console.error('Scraping failed:', error);
    },
  });

  // Handling the film selection
  function handleFilmSelect(imdbId: string | undefined) {
    if (!film?.id || !imdbId) return;

    scrapeFilmMutation.mutate({ imdbId, databaseId: film.id });
  }

  function onSuccess(values: FormSchema) {
    console.log(values);
  }

  function onError() {
    console.log('error');
  }

  useEffect(() => {
    if (film) {
      // Reset the form with the fetched film data
      reset({
        imdb_id: film?.imdb_id,
        title: film?.title,
        release_date: film?.release_date
          ? new Date(film?.release_date)
          : new Date(), // Ensure the date is a Date object
        plot: film?.plot,
        run_time: film?.run_time,
        has_color: film?.has_color,
        rating: film?.rating,
        has_watched: film?.has_watched,
        left_off_point: film?.left_off_point,
      });
    }
  }, [film]);

  return (
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
        <FormField
          control={form.control}
          name='run_time'
          render={({ field }) => (
            <FormItem>
              <FormLabel>Runtime (secs)</FormLabel>
              <FormControl>
                <Input type='number' placeholder='Set the runtime' {...field} />
              </FormControl>
              <FormDescription>
                Runtime of the motion picture in seconds
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name='rating'
          render={({ field }) => (
            <FormItem>
              <FormLabel>Rating</FormLabel>
              <FormControl>
                <Input
                  type='number'
                  step='0.1'
                  placeholder='IMDB Rating'
                  {...field}
                />
              </FormControl>
              <FormDescription>
                IMDB Rating of the motion picture
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name='left_off_point'
          render={({ field }) => (
            <FormItem>
              <FormLabel>Left off point (secs)</FormLabel>
              <FormControl>
                <Input
                  type='number'
                  step='1'
                  placeholder='Timestamp of where viewing stopped'
                  {...field}
                />
              </FormControl>
              <FormDescription>
                Timestamp of where viewing was stopped
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <br />
        <FormField
          control={form.control}
          name='has_color'
          render={({ field }) => (
            <FormItem>
              <FormControl>
                <div className='flex items-center space-x-2'>
                  <Checkbox
                    checked={field.value}
                    onCheckedChange={field.onChange}
                  />
                  <FormLabel>Has color</FormLabel>
                </div>
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <br />
        <FormField
          control={form.control}
          name='has_watched'
          render={({ field }) => (
            <FormItem className='flex items-center space-x-2'>
              <FormControl>
                <div className='flex items-center space-x-2'>
                  <Checkbox
                    checked={field.value}
                    onCheckedChange={field.onChange}
                  />
                  <FormLabel>Has watched</FormLabel>
                </div>
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <br />

        <Button type='submit' disabled={scrapeFilmMutation.isPending}>
          {scrapeFilmMutation.isPending ? 'Scraping...' : 'Save'}
        </Button>
      </form>
    </Form>
  );
}

export default MainForm;
