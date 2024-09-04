import { createMemoryRouter } from 'react-router-dom';

import Films from './films/Films';

import Layout from './Layout';
import Directories from './directories/Directories';
import Home from './home/Home';
import EditFilm from './films/edit-film/EditFilm';
import Film from './films/film/Film';

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
            path: 'edit/:filmId',
            element: <EditFilm />,
          },
          {
            path: ':filmId',
            element: <Film />,
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
