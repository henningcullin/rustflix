import React from 'react';
import { Skeleton } from '@/components/ui/skeleton';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { cn } from '@/lib/utils';

type CoverProps = {
  id: number;
  loadingText?: string;
  errorText?: string;
  className?: string; // Optional to make it more flexible
};

function Cover({ id, className }: CoverProps): React.ReactElement {
  const {
    data: src,
    error,
    isLoading,
  } = useQuery<string, Error>({
    queryKey: ['cover', id],
    queryFn: () => invoke('get_cover', { id }),
  });

  if (isLoading) {
    return <Skeleton className={cn('w-[375px] h-[525px]', className)} />;
  }

  if (error) {
    return <div>{error.message}</div>;
  }

  return (
    <img
      src={src || ''}
      className={cn('w-[375px] h-[525px] rounded-lg', className)}
    />
  );
}

export default Cover;
