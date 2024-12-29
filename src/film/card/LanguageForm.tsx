import { useState, useMemo, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Cross2Icon } from '@radix-ui/react-icons';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Film, Language } from '@/lib/types';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { toast } from '@/lib/hooks/use-toast';

// Extend the database language with component-specific props
type SelectableLanguage = Language & { isSelected: boolean };

function LanguageForm({ film }: { film: Film | undefined }) {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedLanguages, setSelectedLanguages] = useState<Language[]>(
    film?.languages ?? []
  );
  const queryClient = useQueryClient();

  // Fetch all available languages
  const { data: languages = [] } = useQuery({
    queryKey: ['languages'],
    queryFn: async () => invoke<Language[]>('get_all_languages'),
    staleTime: 1000 * 60 * 5, // Cache for 5 minutes
  });

  // Add language mutation
  const addLanguageMutation = useMutation<void, Error, Language>({
    mutationFn: async (language) =>
      invoke('add_language_to_film', {
        filmId: film?.id,
        languageId: language.id,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['film', film?.id?.toString()],
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
      queryClient.invalidateQueries({ queryKey: ['languages'] });
      toast({
        title: 'Language added',
        description: 'Language successfully added.',
      });
    },
    onError: (error) => {
      console.error(error);
      toast({
        variant: 'destructive',
        title: 'Failed to add the language',
        description: error.message,
      });
    },
  });

  // Remove language mutation
  const removeLanguageMutation = useMutation<void, Error, Language>({
    mutationFn: async (language) =>
      invoke('remove_language_from_film', {
        filmId: film?.id,
        languageId: language.id,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['film', film?.id?.toString()],
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
      queryClient.invalidateQueries({ queryKey: ['languages'] });
      toast({
        title: 'Language removed',
        description: 'Language successfully removed.',
      });
    },
    onError: (error) => {
      console.error(error);
      toast({
        variant: 'destructive',
        title: 'Failed to delete the language',
        description: error.message,
      });
    },
  });

  const handleLanguageClicked = useCallback(
    (language: SelectableLanguage) => {
      if (language.isSelected) return removeLanguage(language);
      setSelectedLanguages((prev) => [...prev, language]);
      addLanguageMutation.mutate(language);
    },
    [selectedLanguages]
  );

  const removeLanguage = useCallback(
    (language: Language) => {
      setSelectedLanguages((prev) => prev.filter((l) => l.id !== language.id));
      removeLanguageMutation.mutate(language);
    },
    [selectedLanguages]
  );

  const filteredLanguages = useMemo<SelectableLanguage[]>(
    () =>
      languages
        .filter((language) =>
          language.name.toLowerCase().includes(searchTerm.toLowerCase())
        )
        .map((language) => ({
          ...language,
          isSelected: selectedLanguages.some((l) => l.id === language.id),
        })),
    [languages, selectedLanguages, searchTerm]
  );

  return (
    <Card className='flex-1'>
      <CardHeader>
        <CardTitle>
          <h2 className='text-2xl font-bold mb-4'>Languages</h2>
        </CardTitle>
      </CardHeader>

      <CardContent>
        {/* Selected Languages Grid */}
        <div className='flex flex-wrap gap-2 mb-4'>
          {selectedLanguages.map((language) => (
            <Badge
              key={language.id}
              variant='default'
              className='text-sm py-1 px-2 select-none'
            >
              {language.name}
              <button
                onClick={() => removeLanguage(language)}
                className='ml-2 text-muted-foreground hover:text-red-600'
                aria-label={`Remove ${language.name}`}
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
              placeholder='Search languages...'
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
            />
          </div>
          <ScrollArea className='h-48'>
            <div className='flex flex-wrap gap-2 mb-4'>
              {filteredLanguages.length > 0 ? (
                filteredLanguages.map((language) => (
                  <Badge
                    key={language.id}
                    variant={language.isSelected ? 'default' : 'secondary'}
                    className='text-sm py-1 px-2 cursor-pointer select-none'
                    onClick={() => handleLanguageClicked(language)}
                  >
                    {language.name}
                  </Badge>
                ))
              ) : (
                <p className='text-muted-foreground'>No languages found</p>
              )}
            </div>
          </ScrollArea>
        </div>
      </CardContent>
    </Card>
  );
}

export default LanguageForm;
