import FilmTable from '@/components/datatables/FilmTable/FilmTable';
import { Button } from '@/components/ui/button';
import { PlusIcon } from '@radix-ui/react-icons';

export default function Films() {
  return (
    <div className='px-4 pt-5 w-full'>
      <div className='w-full pb-16 text-center text-4xl font-bold'>
        <h2>Films</h2>
      </div>
      <div className='w-full inline-flex place-content-center mb-10'>
        <div className='inline-flex gap-8 place-content-center py-4 px-12'>
          <Button onClick={() => {}}>
            <PlusIcon className='h-6 w-6 mr-1' />
            New film
          </Button>
        </div>
      </div>
      <FilmTable />
    </div>
  );
}
