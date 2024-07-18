import { Outlet } from "react-router-dom";
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

function Layout() {
  return (
    <>
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
      <Outlet />
    </>
  );
}

export default Layout;
