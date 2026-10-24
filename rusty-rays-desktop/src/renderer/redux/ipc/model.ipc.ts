import type { DataType } from '@/redux/ipc/index.ts';
import { invoke, ipcApi } from '@/redux/ipc/index.ts';

export const modelsIpcApi = ipcApi.injectEndpoints({
  endpoints: (builder) => ({
    getAllSpheres: builder.query<DataType<'model:getAllSpheres'>, null>({
      queryFn: async () => {
        const channelName = 'model:getAllSpheres';
        return invoke(channelName);
      },
      providesTags: ['model:getAllSpheres'],
    }),
    getAllCones: builder.query<DataType<'model:getAllCones'>, null>({
      queryFn: async () => {
        const channelName = 'model:getAllCones';
        return invoke(channelName);
      },
      providesTags: ['model:getAllCones'],
    }),
    getAllTriangles: builder.query<DataType<'model:getAllTriangles'>, null>({
      queryFn: async () => {
        const channelName = 'model:getAllTriangles';
        return invoke(channelName);
      },
      providesTags: ['model:getAllTriangles'],
    }),
    getAllPolygons: builder.query<DataType<'model:getAllPolygons'>, null>({
      queryFn: async () => {
        const channelName = 'model:getAllPolygons';
        return invoke(channelName);
      },
      providesTags: ['model:getAllPolygons'],
    }),
    loadModelFromFilePath: builder.mutation<
      DataType<'model:InitFromFilePath'>,
      string
    >({
      queryFn: async (inputFilePath) => {
        const channelName = 'model:InitFromFilePath';
        return invoke(channelName, inputFilePath);
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
        return invoke(channelName, fileText);
      },
      invalidatesTags: [
        'model:getAllSpheres',
        'model:getAllCones',
        'model:getAllTriangles',
        'model:getAllPolygons',
        'tracer:GetInstanceUuid',
      ],
    }),
    setModel: builder.mutation<DataType<'model:SetModel'>, string | undefined>({
      queryFn: async (modelUuid) => {
        const channelName = 'model:SetModel';
        return invoke(channelName, modelUuid);
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
