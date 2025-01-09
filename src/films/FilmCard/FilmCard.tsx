import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useParams } from 'react-router-dom';
import InfoTab from './tabs/InfoTab/InfoTab';
import { invoke } from '@tauri-apps/api/tauri';
import { Film } from '@/lib/types';
import { useQuery } from '@tanstack/react-query';

export default function FilmCard() {
  const { filmId } = useParams();

  const {
    data: film,
    isFetching: isFetchingFilm,
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

  if (isFetchingFilm) return <p>Fetching...</p>;

  if (isFilmError) return <p>Error</p>;

  if (!film) return <p>No film found</p>;

  return (
    <div className='px-4 pt-5 w-full'>
      <div className='w-full pb-16 text-center text-4xl font-bold'>
        <h2>Film</h2>
      </div>
      <Tabs defaultValue='info'>
        <div className='w-full inline-flex place-content-center mb-10'>
          <div className='inline-flex gap-8 place-content-center py-4 px-12'>
            <TabsList>
              <TabsTrigger value='info'>Info</TabsTrigger>
              <TabsTrigger value='documents'>Documents</TabsTrigger>
              <TabsTrigger value='genres'>Genres</TabsTrigger>
              <TabsTrigger value='languages'>Languages</TabsTrigger>
              <TabsTrigger value='keywords'>Keywords</TabsTrigger>
              <TabsTrigger value='characters'>Characters</TabsTrigger>
              <TabsTrigger value='directors'>Directors</TabsTrigger>
              <TabsTrigger value='writers'>Writers</TabsTrigger>
            </TabsList>
          </div>
        </div>
        <TabsContent value='info'>
          <InfoTab film={film} />
        </TabsContent>
        <TabsContent value='genres'></TabsContent>
        <TabsContent value='documents'></TabsContent>
        <TabsContent value='languages'></TabsContent>
        <TabsContent value='keywords'></TabsContent>
        <TabsContent value='characters'></TabsContent>
        <TabsContent value='directors'></TabsContent>
        <TabsContent value='writers'></TabsContent>
      </Tabs>
    </div>
  );
}
