import React, { ReactElement, useRef, useState } from 'react';

import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogClose,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { OpenInNewWindowIcon } from '@radix-ui/react-icons';

interface Arguments {
  onSelect: (url: string | undefined) => void;
  filePath: string | undefined;
}

const IMDBPopup: React.FC<Arguments> = ({ onSelect, filePath }) => {
  const iframeRef = useRef<null | HTMLIFrameElement>(null);
  const [open, setOpen] = useState<boolean>(false);

  const fileName = filePath?.split('\\')?.pop();
  const url = `https://www.imdb.com/find/?q=${
    fileName
      ?.replace(/\.\d{4}.*\.(mp4|mkv)$/, '')
      // @ts-ignore
      ?.replaceAll('.', ' ') ?? ''
  }`;

  const handleSelect = () => {
    const selectedUrl = iframeRef?.current?.contentWindow?.location?.href;
    onSelect(selectedUrl);
    setOpen(false);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>
          <OpenInNewWindowIcon />
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Select film</DialogTitle>
        </DialogHeader>
        <button onClick={handleSelect}>Select</button>
        <iframe
          ref={iframeRef}
          src={url}
          title='IMDB'
          style={{ width: '100%', height: '100%', border: 'none' }}
        />
        <DialogFooter>
          <DialogClose asChild>
            <Button type='button' variant='secondary'>
              Close
            </Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default IMDBPopup;
