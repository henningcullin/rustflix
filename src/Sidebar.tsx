import { HamburgerMenuIcon } from "@radix-ui/react-icons";

import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@/components/ui/command";

import { ModeToggle } from "./components/ModeToggle";
import { Link, useNavigate } from "react-router-dom";
import { useState } from "react";

function Sidebar() {
  const [open, setOpen] = useState(false);
  const navigate = useNavigate();

  function travel(url: string) {
    setOpen(false);
    navigate(url);
  }

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger>
        <Button className="relative top-0 right-0">
          <HamburgerMenuIcon />
        </Button>
      </SheetTrigger>
      <SheetContent>
        <SheetHeader>
          <SheetTitle>Menu</SheetTitle>
        </SheetHeader>
        <Command>
          <CommandInput placeholder="Type a command or search..." />
          <CommandList>
            <CommandEmpty>No results found.</CommandEmpty>
            <CommandGroup heading="Views">
              <CommandItem onSelect={() => travel("/")}>Home</CommandItem>
              <CommandItem onSelect={() => travel("/directories")}>
                Directories
              </CommandItem>
              <CommandItem onSelect={() => travel("/films")}>Films</CommandItem>
            </CommandGroup>
            <CommandSeparator />
            <CommandGroup heading="Settings">
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
