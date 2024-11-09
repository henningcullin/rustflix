import '../app/globals.css';
import '../app/custom.css';

import { ThemeProvider } from './components/ThemeProvider';
import { RouterProvider } from 'react-router-dom';
import router from './Router';
import useFullscreen from './hooks/use-fullscreen';

function App() {
  useFullscreen();
  return (
    <ThemeProvider>
      <RouterProvider router={router} />
    </ThemeProvider>
  );
}

export default App;
