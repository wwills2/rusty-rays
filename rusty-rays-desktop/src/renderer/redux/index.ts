import { configureStore } from '@reduxjs/toolkit';
import { ipcApi } from '@/redux/ipc';

// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
export const store = configureStore({
  reducer: {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    [ipcApi.reducerPath]: ipcApi.reducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware().concat(ipcApi.middleware),
});

export type StoreRootState = ReturnType<typeof store.getState>;
export type StoreDispatch = typeof store.dispatch;
