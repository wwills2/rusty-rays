import { invoke, ipcApi } from '@/redux/ipc/index.ts';
import type { Sphere } from 'rusty-rays-napi-node';

export const modelsIpcApi = ipcApi.injectEndpoints({
  endpoints: (builder) => ({
    getAllSpheres: builder.query<Sphere[], null>({
      queryFn: async () => {
        const result = await invoke('model:getAllSpheres');
        if (result.data) {
          const spheres: Sphere[] = result.data;
          return { data: spheres };
        } else {
          throw new Error(JSON.stringify(result.error));
        }
      },
    }),
    loadModelFromFile: builder.mutation<boolean, string>({
      queryFn: async (inputFilePath) => {
        const result = await invoke('model:InitFromFile', inputFilePath);
        if (result.data) {
          return { data: true };
        } else {
          throw new Error(JSON.stringify(result.error));
        }
      },
    }),
  }),
  overrideExisting: false,
});

export const { useGetAllSpheresQuery, useLoadModelFromFileMutation } =
  modelsIpcApi;
