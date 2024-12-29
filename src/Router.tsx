import { createMemoryRouter } from 'react-router-dom';

import Films from './film/table/filmtable';

import Layout from './Layout';
import Directories from './directory/table/Directories';
import Home from './home/Home';
import EditFilm from './film/card/+page';
import FilmPage from './film/display/+page';

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
            element: <FilmPage />,
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
