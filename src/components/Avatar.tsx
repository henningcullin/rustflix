import React from 'react';
import { Skeleton } from '@/components/ui/skeleton';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';

interface AvatarProps {
  id: number;
}

function Avatar({ id }: AvatarProps): React.ReactElement {
  const {
    data: src,
    error,
    isLoading,
  } = useQuery<string, Error>({
    queryKey: ['avatar', id],
    queryFn: () => invoke('get_avatar', { id }),
  });

  if (isLoading) {
    return <Skeleton className='rounded-full w-[64px] h-[64px]' />;
  }

  if (error) {
    return <div className='rounded-full w-[64px] h-[64px]' />;
  }

  return <img src={src || ''} className='rounded-full w-[64px] h-[64px]' />;
}

export default Avatar;
