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
import {
  CalendarIcon,
  FileIcon,
  IdCardIcon,
  InputIcon,
  ListBulletIcon,
  PersonIcon,
} from '@radix-ui/react-icons';
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
import { Checkbox } from '@/components/ui/checkbox';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from '@/components/ui/accordion';
import { Badge } from '@/components/ui/badge';
import CharacterTable from './CharacterTable';

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
    },
  });

  const { reset } = form;

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
  }, [film, reset]);

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
      queryClient.invalidateQueries({ queryKey: ['persons'] });
    },
    onError: (error) => {
      console.error('Scraping failed:', error);
    },
  });

  // Handling the film selection
  function handleFilmSelect(imdbId: string | undefined) {
    if (!filmId || !imdbId) return;
    const databaseId = parseInt(filmId);
    if (isNaN(databaseId)) return;

    scrapeFilmMutation.mutate({ imdbId, databaseId });
  }

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

      <Tabs defaultValue='mainForm'>
        <div className='grid place-items-center'>
          <TabsList>
            <TabsTrigger value='mainForm'>
              <InputIcon className='mr-2' />
              Main form
            </TabsTrigger>
            <TabsTrigger value='categories'>
              <ListBulletIcon className='mr-2' />
              Categories
            </TabsTrigger>
            <TabsTrigger value='credits'>
              <PersonIcon className='mr-2' />
              Credits
            </TabsTrigger>
          </TabsList>
        </div>
        <TabsContent value='mainForm'>
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
                    <FormDescription>
                      The id of the film on IMDB
                    </FormDescription>
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
                    <FormDescription>
                      Title of the motion picture
                    </FormDescription>
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
                    <FormDescription>
                      Plot of the motion picture
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
              <FormField
                control={form.control}
                name='run_time'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Runtime (secs)</FormLabel>
                    <FormControl>
                      <Input
                        type='number'
                        placeholder='Set the runtime'
                        {...field}
                      />
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
        </TabsContent>
        <TabsContent value='categories' className='container mx-auto'>
          Here we will handle:
          <ol>
            <li>Languages</li>
            <li>Genres</li>
            <li>Keywords</li>
          </ol>
          <div>
            {film?.keywords?.map((keyword) => (
              <Badge>{keyword}</Badge>
            ))}
          </div>
        </TabsContent>
        <TabsContent value='credits' className='container mx-auto'>
          <Accordion type='single' collapsible>
            <AccordionItem value='item-1'>
              <AccordionTrigger>Characters</AccordionTrigger>
              <AccordionContent>
                <CharacterTable film={film}></CharacterTable>
              </AccordionContent>
            </AccordionItem>
          </Accordion>
          <Accordion type='single' collapsible>
            <AccordionItem value='item-1'>
              <AccordionTrigger>Directors</AccordionTrigger>
              <AccordionContent>
                <ul>
                  {film?.directors.map((director, index, directors) => (
                    <li>
                      <h4>{director.name}</h4>
                      <i>IMDB Id {director.imdb_id}</i>
                      {index < directors?.length - 1 && <Separator></Separator>}
                    </li>
                  ))}
                </ul>
              </AccordionContent>
            </AccordionItem>
          </Accordion>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default EditFilm;
