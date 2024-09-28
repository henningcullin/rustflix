import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { toast } from '@/hooks/use-toast';
import { Film } from '@/lib/types';
import { Cross2Icon } from '@radix-ui/react-icons';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { useCallback, useState } from 'react';

function KeywordForm({ film }: { film: Film | undefined }) {
  const [newKeyword, setNewKeyword] = useState('');

  const queryClient = useQueryClient();

  const createKeywordMutation = useMutation<void, Error, { keyword: string }>({
    mutationFn: async ({ keyword }) => {
      return invoke<void>('create_keyword', { filmId: film?.id, keyword });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['film', film?.id?.toString()],
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
      toast({
        title: 'Keyword created',
        description: `Keyword was successfully created`,
      });
    },
    onError: (error) => {
      console.error(error);
      toast({
        variant: 'destructive',
        title: 'Failed to create the keyword',
        description: error.message,
      });
    },
  });

  const deleteKeywordMutation = useMutation<void, Error, { keyword: string }>({
    mutationFn: async ({ keyword }) => {
      return invoke<void>('delete_keyword', { filmId: film?.id, keyword });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['film', film?.id?.toString()],
      });
      queryClient.invalidateQueries({ queryKey: ['films'] });
      toast({
        title: 'Keyword deleted',
        description: `Keyword was successfully deleted`,
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

  const isUnique = useCallback(
    () => film?.keywords?.includes(newKeyword),
    [film?.keywords, newKeyword]
  );

  const actionDisabled =
    deleteKeywordMutation.isPending ||
    createKeywordMutation.isPending ||
    typeof film?.id !== 'number' ||
    isNaN(film?.id);

  const addDisabled = actionDisabled || newKeyword?.length < 1 || isUnique();

  const addKeyword = useCallback(() => {
    if (addDisabled) return;
    createKeywordMutation.mutate({ keyword: newKeyword });
  }, [film?.id, newKeyword, addDisabled]);

  const deleteKeyword = useCallback(
    (keyword: string) => {
      if (actionDisabled) return;

      deleteKeywordMutation.mutate({ keyword });
    },
    [film?.id, actionDisabled]
  );

  return (
    <div className='max-w-md mx-auto p-6 bg-background rounded-lg shadow'>
      <h2 className='text-2xl font-bold mb-4'>Keywords</h2>
      <div className='mb-4'>
        <div className='flex flex-wrap gap-2'>
          {film?.keywords?.map((keyword) => (
            <Badge
              key={keyword}
              variant='secondary'
              className='text-sm py-1 px-2'
            >
              {keyword}
              <button
                disabled={actionDisabled}
                onClick={() => deleteKeyword(keyword)}
                className='ml-2 text-muted-foreground hover:text-foreground'
                aria-label={`Remove ${keyword} keyword`}
              >
                <Cross2Icon className='h-3 w-3' />
              </button>
            </Badge>
          ))}
        </div>
      </div>
      <div className='flex space-x-2'>
        <Input
          type='text'
          placeholder='Add a keyword'
          value={newKeyword}
          onChange={(e) => setNewKeyword(e.target.value)}
          onKeyPress={(e) => {
            if (e.key === 'Enter') {
              addKeyword();
            }
          }}
          className='flex-grow'
        />
        <Button disabled={addDisabled} onClick={addKeyword}>
          Add
        </Button>
      </div>
    </div>
  );
}

export default KeywordForm;
