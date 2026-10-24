import type { DataType } from '@/redux/ipc/index.ts';
import { invoke, ipcApi } from '@/redux/ipc/index.ts';
import { clearCache, saveLatestRender } from '@/indexed-db-image-cache.ts';
import type { IntersectedObjectInfo } from 'rusty-rays-napi-node/index.ts';

export const tracerIpcApi = ipcApi.injectEndpoints({
  endpoints: (builder) => ({
    render: builder.mutation<DataType<'tracer:TriggerRender'>, null>({
      queryFn: async () => {
        const channelName = 'tracer:TriggerRender';
        return invoke(channelName);
      },
      invalidatesTags: ['tracer:GetRenderStatus', 'tracer:GetRenderImageData'],
    }),

    getRenderStatus: builder.query<DataType<'tracer:GetRenderStatus'>, null>({
      queryFn: async () => {
        const channelName = 'tracer:GetRenderStatus';
        return invoke(channelName);
      },
    }),

    loadRenderImage: builder.query<boolean, string>({
      queryFn: async (instanceUuid) => {
        const channelName = 'tracer:GetRenderImageData';
        const result = await invoke(channelName);

        if (result.error) {
          return { error: result.error };
        }

        const imageData = new Uint8Array(result.data);

        await clearCache();
        await saveLatestRender(instanceUuid, imageData);
        return { data: true };
      },
    }),

    getTracerInstanceUuid: builder.query<string | undefined, null>({
      queryFn: async () => {
        const channelName = 'tracer:GetInstanceUuid';
        const result = await invoke(channelName);

        if (result.error) {
          return { error: result.error };
        }

        const { data: tracerInstanceUuid } = result;
        if (tracerInstanceUuid === 'TRACER_INSTANCE_NOT_LOADED') {
          return { data: undefined };
        } else {
          return { data: tracerInstanceUuid };
        }
      },
      providesTags: ['tracer:GetInstanceUuid'],
    }),

    getIntersectedObjectByPixelPos: builder.query<
      IntersectedObjectInfo | null,
      { x: number; y: number }
    >({
      queryFn: async ({ x, y }) => {
        const channelName = 'tracer:GetIntersectedUuidByPixelPos';
        return  invoke(channelName, x, y);
      },
      providesTags: ['tracer:GetIntersectedUuidByPixelPos'],
    }),
  }),

  overrideExisting: false,
});

export const {
  useRenderMutation,
  useLazyGetIntersectedObjectByPixelPosQuery,
  useGetTracerInstanceUuidQuery,
  useLazyLoadRenderImageQuery,
  useGetRenderStatusQuery,
} = tracerIpcApi;
