import { createMemoryRouter } from 'react-router-dom';

import Films from './films/Films';

import Layout from './Layout';
import Directories from './directories/Directories';
import Home from './home/Home';
import EditFilm from './films/edit-film/EditFilm';

const router = createMemoryRouter([
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
        path: '/film',
        children: [
          {
            path: '/edit/:filmId',
            element: <EditFilm />,
          },
        ],
      },
      {
        path: '/directories',
        element: <Directories />,
      },
    ],
  },
]);

export default router;
