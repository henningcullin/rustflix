import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from '@/components/ui/sidebar';
import { ModeToggle } from '@/components/ModeToggle';
import { useNavigate } from 'react-router-dom';

import { HouseIcon, UserIcon, VideoIcon } from 'lucide-react';
import DirectoryIcon from './lib/icons/DirectoryIcon';

// Menu items.
const items = [
  {
    title: 'Home',
    url: '/',
    icon: <HouseIcon />,
  },
  {
    title: 'Films',
    url: '/films',
    icon: <VideoIcon />,
  },
  {
    title: 'Persons',
    url: '/persons',
    icon: <UserIcon />,
  },
  {
    title: 'Directories',
    url: '/directories',
    icon: <DirectoryIcon />,
  },
];

export default function AppSidebar() {
  const { setOpen } = useSidebar();
  const navigate = useNavigate();

  function openPage(url: string) {
    setOpen(false);
    navigate(url);
  }

  return (
    <Sidebar>
      <SidebarHeader>
        <h1>Rustflix</h1>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Views</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map(({ title, url, icon }) => (
                <SidebarMenuItem key={title} className='cursor-pointer'>
                  <SidebarMenuButton asChild>
                    <a onClick={() => openPage(url)}>
                      {icon}
                      <span>{title}</span>
                    </a>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarGroupLabel>Settings</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <ModeToggle />
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
