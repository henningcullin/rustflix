import { useEffect, useState } from 'react';
import { Film } from '../Films';
import { invoke } from '@tauri-apps/api/tauri';
import { useParams } from 'react-router-dom';

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

import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useForm } from 'react-hook-form';
import { Textarea } from '@/components/ui/textarea';

const formSchema = z.object({
  link: z.string(),
  title: z.string(),
  synopsis: z.string(),
  release_year: z.number(),
  duration: z.number(),
  cover_image: z.string(),
});

type FormSchema = z.infer<typeof formSchema>;

async function getFilm(id: number) {
  try {
    const data = await invoke('get_film', { id });
    console.log(data);
  } catch (error) {
    console.error('Could not get film', error);
  }
}

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

  useEffect(() => {
    if (typeof filmId !== 'string') return;
    const id = parseInt(filmId);
    if (typeof id !== 'number') return;
    getFilm(id);
  }, [filmId]);

  return (
    <div>
      <Form {...form}>
        <form
          onSubmit={form.handleSubmit(onSubmit)}
          className='max-w-96 space-y-6 p-5'
        >
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
            name='release_year'
            render={({ field }) => (
              <FormItem>
                <FormLabel>Release Date</FormLabel>
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
          <Button type='submit'>Submit</Button>
        </form>
      </Form>
    </div>
  );
}

export default EditFilm;
