import { forwardRef, memo } from 'react';
import { Skeleton } from '@/components/ui/skeleton';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { PersonIcon } from '@radix-ui/react-icons';

type AvatarProps = {
  id: number;
};

const Avatar = memo(
  forwardRef<HTMLDivElement, AvatarProps>(({ id }, ref) => {
    const {
      data: src,
      error,
      isLoading,
    } = useQuery<string, Error>({
      queryKey: ['avatar', id],
      queryFn: () => invoke('get_avatar', { id }),
    });

    // Define a base class for the avatar
    const baseClass =
      'rounded-full w-[64px] h-[64px] flex items-center justify-center bg-gray-200';

    // Container for the avatar, skeleton, or error icon
    return (
      <div ref={ref} className={baseClass}>
        {isLoading ? (
          <Skeleton className='rounded-full w-full h-full' />
        ) : error ? (
          <PersonIcon className='w-1/2 h-1/2 text-gray-500' /> // Icon for error state
        ) : (
          <img
            src={src || ''}
            alt='User Avatar'
            className='rounded-full object-cover w-full h-full'
          />
        )}
      </div>
    );
  })
);

export default memo(Avatar);
