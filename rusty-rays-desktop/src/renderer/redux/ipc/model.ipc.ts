import type { DataType } from '@/redux/ipc/index.ts';
import { invoke, ipcApi, processIpcResult } from '@/redux/ipc/index.ts';

export const modelsIpcApi = ipcApi.injectEndpoints({
  endpoints: (builder) => ({
    getAllSpheres: builder.query<DataType<'model:getAllSpheres'>, null>({
      queryFn: async () => {
        const result = await invoke('model:getAllSpheres');
        return processIpcResult(result, (data) => data);
      },
    }),
    loadModelFromFilePath: builder.mutation<
      DataType<'model:InitFromFilePath'>,
      string
    >({
      queryFn: async (inputFilePath) => {
        const result = await invoke('model:InitFromFilePath', inputFilePath);
        return processIpcResult(result, (data) => data);
      },
    }),
    loadModelFromFile: builder.mutation<
      DataType<'model:InitFromFileTextString'>,
      string
    >({
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
