import { invoke, ipcApi, processIpcResult } from '@/redux/ipc/index.ts';
import type { Sphere } from 'rusty-rays-napi-node';

export const modelsIpcApi = ipcApi.injectEndpoints({
  endpoints: (builder) => ({
    getAllSpheres: builder.query<Sphere[], null>({
      queryFn: async () => {
        const result = await invoke('model:getAllSpheres');
        return processIpcResult(result, (data) => data);
      },
    }),
    loadModelFromFilePath: builder.mutation<boolean, string>({
      queryFn: async (inputFilePath) => {
        const result = await invoke('model:InitFromFilePath', inputFilePath);
        return processIpcResult(result, (data) => data);
      },
    }),
    loadModelFromFile: builder.mutation<boolean, string>({
      queryFn: async (fileText) => {
        const result = await invoke('model:InitFromFileTextString', fileText);
        return processIpcResult(result, (data) => data);
      },
    }),
  }),
  overrideExisting: false,
});

export const {
  useGetAllSpheresQuery,
  useLoadModelFromFilePathMutation,
  useLoadModelFromFileMutation,
} = modelsIpcApi;
