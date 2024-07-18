import { createBrowserRouter } from "react-router-dom";

import { Films } from "./films/Films";

import Layout from "./Layout";
import Directories from "./directories/Directories";

const router = createBrowserRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      {
        path: "/",
        element: <Films />,
      },
      {
        path: "/directories",
        element: <Directories />,
      },
    ],
  },
]);

export default router;
