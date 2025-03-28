import { createMemoryRouter } from 'react-router-dom';

import Films from './film/table/+page';

import Layout from './Layout';
import Directories from './directory/table/+page';
import Home from './home/Home';
import EditFilm from './film/card/FilmCard';
import FilmPage from './film/display/+page';
import FilmCard from './films/FilmCard/FilmCard';

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
            path: 'card/:filmId',
            element: <FilmCard />,
          },
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
