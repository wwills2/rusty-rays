import { AppRouter } from '@/routes/Router.tsx';
import { useEffect } from 'react';

function App() {
  useEffect(() => {
    // set the theme to light
    document.documentElement.classList.remove('dark'); // enable light
  }, []);

  return (
    <div className="h-full w-full">
      <AppRouter />
    </div>
  );
}

export default App;
