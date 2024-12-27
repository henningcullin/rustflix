import { Outlet } from 'react-router-dom';
import AppSidebar from './AppSidebar';
import { Toaster } from './components/ui/toaster';
import { SidebarProvider, SidebarTrigger } from './components/ui/sidebar';

function Layout() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarTrigger />
      <Outlet />
      <Toaster />
    </SidebarProvider>
  );
}

export default Layout;
