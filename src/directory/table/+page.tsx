import { PlusIcon } from '@radix-ui/react-icons';
import { DirectoryTable } from './DirectoryTable/DirectoryTable';
import { Button } from '@/components/ui/button';
import useDirectoryAdd from './useDirectoryAdd';

function Directories() {
  const { directoryAdd, AddDialog } = useDirectoryAdd();

  return (
    <div className='px-4 pt-5 w-full'>
      <AddDialog />
      <div className='w-full pb-16 text-center text-4xl font-bold'>
        <h2>Directories</h2>
      </div>
      <div className='w-full inline-flex place-content-center mb-10'>
        <div className='inline-flex gap-8 place-content-center py-4 px-12'>
          <Button onClick={directoryAdd}>
            <PlusIcon className='h-6 w-6 mr-1' />
            Add Directory
          </Button>
        </div>
      </div>
      <DirectoryTable></DirectoryTable>
    </div>
  );
}

export default Directories;
