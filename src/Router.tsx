import { createBrowserRouter } from 'react-router-dom';

import Films from './films/Films';

import Layout from './Layout';
import Directories from './directories/Directories';
import Home from './home/Home';

const router = createBrowserRouter([
  {
    path: '/',
    element: <Layout />,
    children: [
      {
        path: '/',
        element: <Home />,
      },
      {
        path: '/films',
        element: <Films />,
      },
      {
        path: '/directories',
        element: <Directories />,
      },
    ],
  },
]);

export default router;
