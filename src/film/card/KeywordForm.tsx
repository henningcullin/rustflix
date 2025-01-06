import { useState, useMemo, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Cross2Icon } from '@radix-ui/react-icons';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Film, Keyword } from '@/lib/types';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { toast } from '@/lib/hooks/use-toast';

// Extend the database keyword with component-specific props
type SelectableKeyword = Keyword & { isSelected: boolean };

function KeywordForm({ film }: { film: Film | undefined }) {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedKeywords, setSelectedKeywords] = useState<Keyword[]>(
    film?.keywords ?? []
  );
  const queryClient = useQueryClient();

  // Fetch all available keywords
  const { data: keywords = [] } = useQuery({
    queryKey: ['keywords'],
    queryFn: async () => invoke<Keyword[]>('get_all_keywords'),
  });

  // Add keyword mutation
  const addKeywordMutation = useMutation<void, Error, Keyword>({
    mutationFn: async (keyword) =>
      invoke('add_keyword_to_film', {
        filmId: film?.id,
        keywordId: keyword.id,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['film', film?.id?.toString()],
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
      toast({
        title: 'Keyword added',
        description: 'Keyword successfully added.',
      });
    },
    onError: (error) => {
      console.error(error);
      toast({
        variant: 'destructive',
        title: 'Failed to add the keyword',
        description: error.message,
      });
    },
  });

  // Remove keyword mutation
  const removeKeywordMutation = useMutation<void, Error, Keyword>({
    mutationFn: async (keyword) =>
      invoke('remove_keyword_from_film', {
        filmId: film?.id,
        keywordId: keyword.id,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['film', film?.id?.toString()],
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
      toast({
        title: 'Keyword removed',
        description: 'Keyword successfully removed.',
      });
    },
    onError: (error) => {
      console.error(error);
      toast({
        variant: 'destructive',
        title: 'Failed to delete the keyword',
        description: error.message,
      });
    },
  });

  const handleKeywordClicked = useCallback(
    (keyword: SelectableKeyword) => {
      if (keyword.isSelected) return removeKeyword(keyword);
      setSelectedKeywords((prev) => [...prev, keyword]);
      addKeywordMutation.mutate(keyword);
    },
    [selectedKeywords]
  );

  const removeKeyword = useCallback(
    (keyword: Keyword) => {
      setSelectedKeywords((prev) => prev.filter((k) => k.id !== keyword.id));
      removeKeywordMutation.mutate(keyword);
    },
    [selectedKeywords]
  );

  const filteredKeywords = useMemo<SelectableKeyword[]>(
    () =>
      keywords
        .filter((keyword) =>
          keyword.name.toLowerCase().includes(searchTerm.toLowerCase())
        )
        .map((keyword) => ({
          ...keyword,
          isSelected: selectedKeywords.some((k) => k.id === keyword.id),
        })),
    [keywords, selectedKeywords, searchTerm]
  );

  return (
    <Card className='flex-1'>
      <CardHeader>
        <CardTitle>
          <h2 className='text-2xl font-bold mb-4'>Keywords</h2>
        </CardTitle>
      </CardHeader>

      <CardContent>
        {/* Selected Keywords Grid */}
        <div className='flex flex-wrap gap-2 mb-4'>
          {selectedKeywords.map((keyword) => (
            <Badge
              key={keyword.id}
              variant='default'
              className='text-sm py-1 px-2 select-none'
            >
              {keyword.name}
              <button
                onClick={() => removeKeyword(keyword)}
                className='ml-2 text-muted-foreground hover:text-red-600'
                aria-label={`Remove ${keyword.name}`}
              >
                <Cross2Icon className='h-3 w-3' />
              </button>
            </Badge>
          ))}
        </div>

        <div className='p-1 w-full'>
          <div className='mb-2'>
            <Input
              type='text'
              placeholder='Search keywords...'
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
            />
          </div>
          <ScrollArea className='h-48'>
            <div className='flex flex-wrap gap-2 mb-4'>
              {filteredKeywords.length > 0 ? (
                filteredKeywords.map((keyword) => (
                  <Badge
                    key={keyword.id}
                    variant={keyword.isSelected ? 'default' : 'secondary'}
                    className='text-sm py-1 px-2 cursor-pointer select-none'
                    onClick={() => handleKeywordClicked(keyword)}
                  >
                    {keyword.name}
                  </Badge>
                ))
              ) : (
                <p className='text-muted-foreground'>No keywords found</p>
              )}
            </div>
          </ScrollArea>
        </div>
      </CardContent>
    </Card>
  );
}

export default KeywordForm;
