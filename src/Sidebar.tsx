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
import { Link } from "react-router-dom";

function Sidebar() {
  return (
    <Sheet>
      <SheetTrigger>
        <Button className="relative top-0 right-0">
          <HamburgerMenuIcon />
        </Button>
      </SheetTrigger>
      <SheetContent>
        <SheetHeader>
          <SheetTitle>Menu</SheetTitle>
        </SheetHeader>
        <ModeToggle />
        <Link to="/directories">Directories</Link>
      </SheetContent>
    </Sheet>
  );
}

export default Sidebar;
