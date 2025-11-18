import { createApi } from '@reduxjs/toolkit/query/react';
import { fakeBaseQuery } from '@reduxjs/toolkit/query';

export const ipcApi = createApi({
  reducerPath: 'ipcApi',
  endpoints: () => ({}),
  baseQuery: fakeBaseQuery(),
});
