import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import '@/index.css';
import App from './App.tsx';
import { Provider as ReduxStoreProvider } from 'react-redux';
import { store } from '@/redux';

const root = document.getElementById('root');

if (root) {
  createRoot(root).render(
    <StrictMode>
      <ReduxStoreProvider store={store}>
        <App />
      </ReduxStoreProvider>
    </StrictMode>,
  );
} else {
  throw new Error('failed to find root element');
}
