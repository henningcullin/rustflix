import { DirectoryDialog } from "./DirectoryDialog";
import { DirectoryTable } from "./DirectoryTable";
import { Button } from "@/components/ui/button";

export interface Directory {
  id: number;
  path: string;
}

function Directories() {
  return (
    <>
      <div className="pt-12">
        <DirectoryDialog>
          <Button>Add Directory</Button>
        </DirectoryDialog>
        <DirectoryTable></DirectoryTable>
      </div>
    </>
  );
}

export default Directories;