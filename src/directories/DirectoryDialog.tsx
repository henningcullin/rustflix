import { directoryAtom } from '@/lib/atoms';

import { invoke } from '@tauri-apps/api/tauri';
import { ReactElement, useState } from 'react';

import { useAtom } from 'jotai';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Directory } from './Directories';

export function DirectoryDialog({ children }: { children: ReactElement }) {
  const [directory, setDirectory] = useState<string>('');
  const [directories, setDirectories] = useAtom(directoryAtom);

  async function addDirectory(path: string) {
    try {
      const dir: Directory | null = await invoke('add_directory', { path });
      if (dir) {
        setDirectory('');
        setDirectories([...directories, dir]);
      }
    } catch (error) {
      console.error('Failed to add directory', error);
    }
  }

  async function selectDirectory() {
    try {
      const path: string | null = await invoke('select_directory');
      if (typeof path === 'string' && path) setDirectory(path);
    } catch (error) {
      console.error('Failed to select directory', error);
    }
  }

  return (
    <Dialog>
      <DialogTrigger>{children}</DialogTrigger>
      <DialogContent>
        <DialogHeader></DialogHeader>
        <div style={{ width: '450px' }}>
          <Label>Directory</Label>
          <div style={{ display: 'flex', gap: '5px' }}>
            <Input value={directory} readOnly disabled />
            <Button onClick={selectDirectory}>Select Directory</Button>
          </div>
          <Button onClick={() => addDirectory(directory)}>Add Directory</Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
