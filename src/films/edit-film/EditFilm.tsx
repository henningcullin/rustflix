import { useEffect, useState } from 'react';
import { Film } from '../Films';
import { invoke } from '@tauri-apps/api/tauri';
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
import { CalendarIcon, OpenInNewWindowIcon } from '@radix-ui/react-icons';
import { Calendar } from '@/components/ui/calendar';
import SelectFilmPopup from './SelectFilmPopup';

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
  const [film, setFilm] = useState<Film>();

  const { filmId } = useParams();

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      title: '',
    },
  });

  function onSubmit(values: FormSchema) {
    console.log(values);
  }

  async function getFilm(id: number) {
    try {
      const data: Film | undefined = await invoke('get_film', { id });
      if (data) setFilm(data);
    } catch (error) {
      console.error('Could not get film', error);
    }
  }

  useEffect(() => {
    if (typeof filmId !== 'string') return;
    const id = parseInt(filmId);
    if (typeof id !== 'number') return;
    getFilm(id);
  }, [filmId]);

  return (
    <div>
      <SelectFilmPopup
        onSelect={(value) => console.log(value)}
        filePath={film?.file}
      />
      <Form {...form}>
        <form
          onSubmit={form.handleSubmit(onSubmit)}
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
          ></FormField>
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
          <Button type='submit'>Submit</Button>
        </form>
      </Form>
    </div>
  );
}

export default EditFilm;
