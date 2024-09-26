import { useParams } from 'react-router-dom';
import { getFilmName } from '@/lib/utils';
import {
  FileIcon,
  IdCardIcon,
  InputIcon,
  ListBulletIcon,
  PersonIcon,
} from '@radix-ui/react-icons';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { Film } from '@/lib/types';
import { Separator } from '@/components/ui/separator';
import DirectoryIcon from '@/components/icons/DirectoryIcon';
import ValueDisplay from './ValueDisplay';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from '@/components/ui/accordion';
import { Badge } from '@/components/ui/badge';
import CharacterTable from './CharacterTable/CharacterTable';
import DirectorTable from './DirectorTable/DirectorTable';
import MainForm from './MainForm';

function EditFilm() {
  const { filmId } = useParams();

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
          <MainForm film={film} />
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
              <Badge key={keyword}>{keyword}</Badge>
            ))}
          </div>
        </TabsContent>
        <TabsContent value='credits' className='container mx-auto'>
          <Accordion type='single' collapsible>
            <AccordionItem value='item-1'>
              <AccordionTrigger>Characters</AccordionTrigger>
              <AccordionContent>
                <CharacterTable film={film} />
              </AccordionContent>
            </AccordionItem>
          </Accordion>
          <Accordion type='single' collapsible>
            <AccordionItem value='item-1'>
              <AccordionTrigger>Directors</AccordionTrigger>
              <AccordionContent>
                <DirectorTable film={film} />
              </AccordionContent>
            </AccordionItem>
          </Accordion>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default EditFilm;
