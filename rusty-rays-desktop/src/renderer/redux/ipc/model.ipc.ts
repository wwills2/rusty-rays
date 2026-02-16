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
    getAllCones: builder.query<DataType<'model:getAllCones'>, null>({
      queryFn: async () => {
        const channelName = 'model:getAllCones';
        const result = await invoke(channelName);
        return processIpcResult(channelName, result, (data) => data);
      },
      providesTags: ['model:getAllCones'],
    }),
    getAllTriangles: builder.query<DataType<'model:getAllTriangles'>, null>({
      queryFn: async () => {
        const channelName = 'model:getAllTriangles';
        const result = await invoke(channelName);
        return processIpcResult(channelName, result, (data) => data);
      },
      providesTags: ['model:getAllTriangles'],
    }),
    getAllPolygons: builder.query<DataType<'model:getAllPolygons'>, null>({
      queryFn: async () => {
        const channelName = 'model:getAllPolygons';
        const result = await invoke(channelName);
        return processIpcResult(channelName, result, (data) => data);
      },
      providesTags: ['model:getAllPolygons'],
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
      invalidatesTags: [
        'model:getAllSpheres',
        'model:getAllCones',
        'model:getAllTriangles',
        'model:getAllPolygons',
      ],
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
      invalidatesTags: [
        'model:getAllSpheres',
        'model:getAllCones',
        'model:getAllTriangles',
        'model:getAllPolygons',
        'tracer:GetInstanceUuid',
        '',
      ],
    }),
    setModel: builder.mutation<DataType<'model:SetModel'>, string | undefined>({
      queryFn: async (modelUuid) => {
        const channelName = 'model:SetModel';
        const result = await invoke(channelName, modelUuid);
        return processIpcResult(channelName, result, (data) => data);
      },
      invalidatesTags: [
        'tracer:GetInstanceUuid',
        'model:getAllSpheres',
        'model:getAllCones',
        'model:getAllTriangles',
        'model:getAllPolygons',
      ],
    }),
  }),
  overrideExisting: false,
});

export const {
  useGetAllSpheresQuery,
  useGetAllConesQuery,
  useGetAllTrianglesQuery,
  useGetAllPolygonsQuery,
  useLoadModelFromFilePathMutation,
  useLoadModelFromFileMutation,
  useSetModelMutation,
} = modelsIpcApi;
