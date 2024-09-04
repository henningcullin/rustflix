import React from 'react';
import { Skeleton } from '@/components/ui/skeleton';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';

interface CoverProps {
  id: number;
  loadingText?: string;
  errorText?: string;
}

const Cover: React.FC<CoverProps> = ({ id }) => {
  const {
    data: src,
    error,
    isLoading,
  } = useQuery<string, Error>({
    queryKey: ['cover', id],
    queryFn: () => invoke('get_cover', { id }),
  });

  if (isLoading) {
    return <Skeleton className='w-[375px] h-[525px]' />;
  }

  if (error) {
    return <div>{error.message}</div>;
  }

  return <img src={src || ''} className='w-[375px] h-[525px]' />;
};

export default Cover;
