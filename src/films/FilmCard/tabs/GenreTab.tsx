import { Button } from '@/components/ui/button';
import { Film } from '@/lib/types';

type GenreTabProps = {
  film: Film;
};

export default function GenreTab({ film }: GenreTabProps) {
  return (
    <div className='w-full border-ws rounded-sm'>
      <div className='flex place-content-center w-full mb-4'>
        <div className='inline-flex gap-3'>
          <Button>New</Button>
        </div>
      </div>
    </div>
  );
}
