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
      {/* TODO figure out the store typing and remove this suppression */}
      {/* eslint-disable-next-line*/}
      <ReduxStoreProvider store={store}>
        <App />
      </ReduxStoreProvider>
    </StrictMode>,
  );
} else {
  throw new Error('failed to find root element');
}
