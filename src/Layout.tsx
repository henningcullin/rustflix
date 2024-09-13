import { Outlet } from 'react-router-dom';
import Sidebar from './Sidebar';
import { Toaster } from './components/ui/toaster';

function Layout() {
  return (
    <>
      <Sidebar />
      <Outlet />
      <Toaster />
    </>
  );
}

export default Layout;
