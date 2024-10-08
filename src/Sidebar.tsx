import { HamburgerMenuIcon } from '@radix-ui/react-icons';

import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from '@/components/ui/sheet';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from '@/components/ui/command';

import { ModeToggle } from './components/ModeToggle';
import { useNavigate } from 'react-router-dom';
import { useState } from 'react';

function Sidebar() {
  const [open, setOpen] = useState(false);
  const navigate = useNavigate();

  function travel(url: string) {
    setOpen(false);
    navigate(url);
  }

  const views = [
    { destination: '/', label: 'Home' },
    { destination: '/directories', label: 'Directories' },
    { destination: '/films', label: 'Films' },
  ];

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger className='p-2'>
        <HamburgerMenuIcon className='w-7 h-7' />
      </SheetTrigger>
      <SheetContent side='left'>
        <SheetHeader>
          <SheetTitle>Menu</SheetTitle>
        </SheetHeader>
        <Command>
          <CommandInput placeholder='Type a command or search...' />
          <CommandList>
            <CommandEmpty>No results found.</CommandEmpty>
            <CommandGroup heading='Views'>
              {views.map((view) => (
                <CommandItem
                  onSelect={() => travel(view.destination)}
                  key={view.label}
                >
                  {view.label}
                </CommandItem>
              ))}
            </CommandGroup>
            <CommandSeparator />
            <CommandGroup heading='Settings'>
              <CommandItem>
                <ModeToggle />
              </CommandItem>
            </CommandGroup>
          </CommandList>
        </Command>
      </SheetContent>
    </Sheet>
  );
}

export default Sidebar;
