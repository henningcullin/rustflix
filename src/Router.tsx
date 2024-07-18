import { createBrowserRouter } from "react-router-dom";

import { Films } from "./films/Films";
import { DirectoryTable } from "./directories/DirectoryTable";
import Layout from "./Layout";

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
        element: <DirectoryTable />,
      },
    ],
  },
]);

export default router;
