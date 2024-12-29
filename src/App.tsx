import '../app/globals.css';
import '../app/custom.css';

import { ThemeProvider } from './lib/ThemeProvider';
import { RouterProvider } from 'react-router-dom';
import router from './Router';
import useFullscreen from './lib/hooks/use-fullscreen';

function App() {
  useFullscreen();
  return (
    <ThemeProvider>
      <RouterProvider router={router} />
    </ThemeProvider>
  );
}

export default App;
