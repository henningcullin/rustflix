import { useState, useMemo, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Cross2Icon } from '@radix-ui/react-icons';
import {
  Accordion,
  AccordionItem,
  AccordionTrigger,
  AccordionContent,
} from '@/components/ui/accordion';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Film, Genre } from '@/lib/types';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { toast } from '@/hooks/use-toast';

// Extend the database genre with component specific props
type SelectableGenre = Genre & { isSelected: boolean };

function GenreForm({ film }: { film: Film | undefined }) {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedGenres, setSelectedGenres] = useState<Genre[]>(
    film?.genres ?? []
  );
  const queryClient = useQueryClient();

  // Fetch all available genres
  const { data: genres = [] } = useQuery({
    queryKey: ['genres'],
    queryFn: async () => invoke<Genre[]>('get_all_genres'),
    staleTime: 1000 * 60 * 5, // Cache for 5 minutes
  });

  // Add genre mutation
  const addGenreMutation = useMutation<void, Error, Genre>({
    mutationFn: async (genre) =>
      invoke('add_genre_to_film', { filmId: film?.id, genreId: genre.id }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['film', film?.id?.toString()],
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
      toast({ title: 'Genre added', description: 'Genre successfully added.' });
    },
  });

  // Remove genre mutation
  const removeGenreMutation = useMutation<void, Error, Genre>({
    mutationFn: async (genre) =>
      invoke('remove_genre_from_film', { filmId: film?.id, genreId: genre.id }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['film', film?.id?.toString()],
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
      toast({
        title: 'Genre removed',
        description: 'Genre successfully removed.',
      });
    },
  });

  const handleGenreClicked = useCallback(
    (genre: SelectableGenre) => {
      if (genre.isSelected) return removeGenre(genre);
      setSelectedGenres((prev) => [...prev, genre]);
      addGenreMutation.mutate(genre);
    },
    [selectedGenres]
  );

  const removeGenre = useCallback(
    (genre: Genre) => {
      setSelectedGenres((prev) => prev.filter((g) => g.id !== genre.id));
      removeGenreMutation.mutate(genre);
    },
    [selectedGenres]
  );

  const filteredGenres = useMemo<SelectableGenre[]>(
    () =>
      genres
        .filter((genre) =>
          genre.name.toLowerCase().includes(searchTerm.toLowerCase())
        )
        .map((genre) => ({
          ...genre,
          isSelected: selectedGenres.some((g) => g.id === genre.id),
        })),
    [genres, selectedGenres, searchTerm]
  );

  return (
    <Card>
      <CardHeader>
        <CardTitle>
          <h2 className='text-2xl font-bold mb-4'>Genres</h2>
        </CardTitle>
      </CardHeader>

      <CardContent>
        {/* Selected Genres Grid */}
        <div className='flex flex-wrap gap-2 mb-4'>
          {selectedGenres.map((genre) => (
            <Badge
              key={genre.id}
              variant='secondary'
              className='text-sm py-1 px-2 select-none'
            >
              {genre.name}
              <button
                onClick={() => removeGenre(genre)}
                className='ml-2 text-muted-foreground hover:text-foreground'
                aria-label={`Remove ${genre.name}`}
              >
                <Cross2Icon className='h-3 w-3' />
              </button>
            </Badge>
          ))}
        </div>

        {/* Accordion for Available Genres */}
        <Accordion type='single' collapsible className='w-full'>
          <AccordionItem value='genres'>
            <AccordionTrigger className='flex justify-between items-center py-2 select-none'>
              <span>Select Genres</span>
            </AccordionTrigger>
            <AccordionContent className='p-1'>
              <div className='mb-2'>
                <Input
                  type='text'
                  placeholder='Search genres...'
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                />
              </div>
              <ScrollArea className='max-h-60'>
                <div className='flex flex-wrap gap-2 mb-4'>
                  {filteredGenres.length > 0 ? (
                    filteredGenres.map((genre) => (
                      <Badge
                        key={genre.id}
                        variant={genre.isSelected ? 'default' : 'secondary'}
                        className='text-sm py-1 px-2 cursor-pointer select-none'
                        onClick={() => handleGenreClicked(genre)}
                      >
                        {genre.name}
                      </Badge>
                    ))
                  ) : (
                    <p className='text-muted-foreground'>No genres found</p>
                  )}
                </div>
              </ScrollArea>
            </AccordionContent>
          </AccordionItem>
        </Accordion>
      </CardContent>
    </Card>
  );
}

export default GenreForm;
