import { Outlet } from 'react-router-dom';
import AppSidebar from './AppSidebar';
import { Toaster } from '@/components/ui/toaster';
import {
  SidebarProvider,
  SidebarTrigger,
  useSidebar,
} from '@/components/ui/sidebar';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { DeleteFilmProvider } from './films/FilmCard/tabs/InfoTab/actions/useDeleteFilm';

export default function Layout() {
  return (
    <SidebarProvider defaultOpen={false}>
      <DeleteFilmProvider>
        <AppSidebar />
        <CustomSidebarTrigger />
        <Outlet />
        <Toaster />
      </DeleteFilmProvider>
    </SidebarProvider>
  );
}

function CustomSidebarTrigger() {
  const { open } = useSidebar();

  const sidebarTooltip = open ? 'Close sidebar' : 'Open sidebar';

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <SidebarTrigger className='mt-2 ml-2 h-8 w-8' />
        </TooltipTrigger>
        <TooltipContent>
          <p>{sidebarTooltip}</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
