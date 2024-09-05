import { DirectoryDialog } from './DirectoryDialog';
import { DirectoryTable } from './DirectoryTable';
import { Button } from '@/components/ui/button';

function Directories() {
  return (
    <>
      <div className='pt-12 p-4'>
        <DirectoryDialog>
          <Button>Add Directory</Button>
        </DirectoryDialog>
      </div>
      <DirectoryTable></DirectoryTable>
    </>
  );
}

export default Directories;
