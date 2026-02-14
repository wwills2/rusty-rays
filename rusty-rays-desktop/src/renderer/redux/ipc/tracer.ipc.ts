import type { DataType } from '@/redux/ipc/index.ts';
import { invoke, ipcApi, processIpcResult } from '@/redux/ipc/index.ts';
import { clearCache, saveLatestRender } from '@/indexed-db-image-cache.ts';

export const tracerIpcApi = ipcApi.injectEndpoints({
  endpoints: (builder) => ({
    render: builder.mutation<DataType<'tracer:TriggerRender'>, null>({
      queryFn: async () => {
        const channelName = 'tracer:TriggerRender';
        const result = await invoke(channelName);
        return processIpcResult(channelName, result, (data) => data);
      },
      invalidatesTags: ['tracer:GetRenderStatus', 'tracer:GetRenderImageData'],
    }),

    getRenderStatus: builder.query<DataType<'tracer:GetRenderStatus'>, null>({
      queryFn: async () => {
        const channelName = 'tracer:GetRenderStatus';
        const result = await invoke(channelName);
        return processIpcResult(channelName, result, (data) => data);
      },
    }),

    loadRenderImage: builder.query<boolean, string>({
      queryFn: async (instanceUuid) => {
        const channelName = 'tracer:GetRenderImageData';
        const result = await invoke(channelName);

        const { data: imageData } = processIpcResult(
          channelName,
          result,
          (data) => new Uint8Array(data),
        );
        await clearCache();
        await saveLatestRender(instanceUuid, imageData);
        return { data: true };
      },
    }),

    getTracerInstanceUuid: builder.query<string | undefined, null>({
      queryFn: async () => {
        const channelName = 'tracer:GetInstanceUuid';
        const result = await invoke(channelName);
        const { data: tracerInstanceUuid } = processIpcResult(
          channelName,
          result,
          (data) => data,
        );
        if (tracerInstanceUuid === 'TRACER_INSTANCE_NOT_LOADED') {
          return { data: undefined };
        } else {
          return { data: tracerInstanceUuid };
        }
      },
      providesTags: ['tracer:GetInstanceUuid'],
    }),

    getIntersectedUuidByPixelPos: builder.query<
      string | null,
      { x: number; y: number }
    >({
      queryFn: async ({ x, y }) => {
        const channelName = 'tracer:GetIntersectedUuidByPixelPos';
        const result = await invoke(channelName, x, y);
        return processIpcResult(channelName, result, (data) => data);
      },
      providesTags: ['tracer:GetIntersectedUuidByPixelPos'],
    }),
  }),

  overrideExisting: false,
});

export const {
  useRenderMutation,
  useLazyGetIntersectedUuidByPixelPosQuery,
  useGetTracerInstanceUuidQuery,
  useLazyLoadRenderImageQuery,
  useGetRenderStatusQuery,
} = tracerIpcApi;
