import '../app/globals.css';
import '../app/custom.css';

import { ThemeProvider } from './components/ThemeProvider';
import { RouterProvider } from 'react-router-dom';
import router from './Router';

function App() {
  return (
    <ThemeProvider>
      <RouterProvider router={router} />
    </ThemeProvider>
  );
}

export default App;
