import type { DataType } from '@/redux/ipc/index.ts';
import { invoke, ipcApi, processIpcResult } from '@/redux/ipc/index.ts';

export const modelsIpcApi = ipcApi.injectEndpoints({
  endpoints: (builder) => ({
    getAllSpheres: builder.query<DataType<'model:getAllSpheres'>, null>({
      queryFn: async () => {
        const channelName = 'model:getAllSpheres';
        const result = await invoke(channelName);
        return processIpcResult(channelName, result, (data) => data);
      },
      providesTags: ['model:getAllSpheres'],
    }),
    loadModelFromFilePath: builder.mutation<
      DataType<'model:InitFromFilePath'>,
      string
    >({
      queryFn: async (inputFilePath) => {
        const channelName = 'model:InitFromFilePath';
        const result = await invoke(channelName, inputFilePath);
        return processIpcResult(channelName, result, (data) => data);
      },
      invalidatesTags: ['model:getAllSpheres'],
    }),
    loadModelFromFile: builder.mutation<
      DataType<'model:InitFromFileTextString'>,
      string
    >({
      queryFn: async (fileText) => {
        const channelName = 'model:InitFromFileTextString';
        const result = await invoke(channelName, fileText);
        return processIpcResult(channelName, result, (data) => data);
      },
      invalidatesTags: ['model:getAllSpheres', 'tracer:GetInstanceUuid', ''],
    }),
    setModel: builder.mutation<DataType<'model:SetModel'>, string | undefined>({
      queryFn: async (modelUuid) => {
        const channelName = 'model:SetModel';
        const result = await invoke(channelName, modelUuid);
        return processIpcResult(channelName, result, (data) => data);
      },
      invalidatesTags: ['model:getAllSpheres', 'tracer:GetInstanceUuid'],
    }),
  }),
  overrideExisting: false,
});

export const {
  useGetAllSpheresQuery,
  useLoadModelFromFilePathMutation,
  useLoadModelFromFileMutation,
  useSetModelMutation,
} = modelsIpcApi;
