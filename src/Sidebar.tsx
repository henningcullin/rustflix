import { HamburgerMenuIcon } from "@radix-ui/react-icons";

import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { ModeToggle } from "./components/ModeToggle";
import { Button } from "./components/ui/button";

function Sidebar() {
  return (
    <Sheet>
      <SheetTrigger>
        <Button>
          <HamburgerMenuIcon />
        </Button>
      </SheetTrigger>
      <SheetContent>
        <SheetHeader>
          <SheetTitle>Menu</SheetTitle>
        </SheetHeader>

        <ModeToggle />
      </SheetContent>
    </Sheet>
  );
}

export default Sidebar;
