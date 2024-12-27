import { useQuery } from '@tanstack/react-query';
import { CaretSortIcon, CheckIcon } from '@radix-ui/react-icons';
import { Button } from './ui/button';
import { Popover, PopoverContent, PopoverTrigger } from './ui/popover';
import { cn } from '@/components/lib/utils';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from './ui/command';
import { invoke } from '@tauri-apps/api/tauri';
import { Person } from '@/components/lib/types';
import { Avatar, AvatarFallback, AvatarImage } from './ui/avatar';
import { useMemo, useState } from 'react';
import { ScrollArea } from './ui/scroll-area';
import { CommandLoading } from 'cmdk';

function PersonAvatar({ person }: { person: Person }) {
  const { data: src } = useQuery<string, Error>({
    queryKey: ['avatar', person.id.toString()],
    queryFn: () => invoke('get_avatar', { id: person.id }),
  });

  const initials = useMemo(() => {
    const nameSplit = person?.name?.split(/[\s-]+/);
    const firstInitial = nameSplit?.at(0)?.at(0)?.toUpperCase();
    const secondInitial = nameSplit?.at(1)?.at(0)?.toUpperCase();
    return `${firstInitial ?? ''} ${secondInitial ?? ''}`;
  }, [person]);

  return (
    <Avatar>
      <AvatarImage src={src} />
      <AvatarFallback>{initials}</AvatarFallback>
    </Avatar>
  );
}

export function PersonBox({
  value,
  onChange,
}: {
  value: number | string;
  onChange: (newValue: number) => void;
}) {
  const [open, setOpen] = useState<boolean>(false);

  // Fetch persons using TanStack Query
  const { data: persons = [], isLoading } = useQuery({
    queryKey: ['persons'],
    queryFn: async () => {
      return await invoke<Person[]>('get_all_persons');
    },
  });

  const selectedPerson = useMemo(
    () => persons.find((person) => person.id == value),
    [value, persons]
  );

  return (
    <Popover open={open} onOpenChange={setOpen} modal={true}>
      <PopoverTrigger asChild>
        <Button
          variant='outline'
          role='combobox'
          aria-label='Select person'
          aria-expanded={open}
          className={cn(
            'w-full justify-between h-[50px]',
            !value && 'text-muted-foreground'
          )}
        >
          {selectedPerson ? (
            <>
              <PersonAvatar person={selectedPerson} />
              <span className='ml-2'>{selectedPerson.name}</span>
            </>
          ) : (
            'Select person'
          )}
          <CaretSortIcon className='ml-2 h-4 w-4 shrink-0 opacity-50' />
        </Button>
      </PopoverTrigger>
      <PopoverContent className='p-0'>
        <Command>
          <CommandInput placeholder='Search person...' className='h-9' />
          <CommandList className='overflow-hidden'>
            {isLoading ? (
              <CommandLoading>Loading people</CommandLoading>
            ) : (
              <ScrollArea className='h-72'>
                <CommandEmpty>No person found.</CommandEmpty>
                <CommandGroup>
                  {persons.map((person) => (
                    <CommandItem
                      value={person.name}
                      key={person.id}
                      onSelect={() => {
                        setOpen(false);
                        onChange(person.id);
                      }}
                    >
                      <PersonAvatar person={person} />
                      <span className='ml-2'>{person.name}</span>
                      <CheckIcon
                        className={cn(
                          'ml-auto h-4 w-4',
                          person.id == value ? 'opacity-100' : 'opacity-0'
                        )}
                      />
                    </CommandItem>
                  ))}
                </CommandGroup>
              </ScrollArea>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}

export default PersonBox;
