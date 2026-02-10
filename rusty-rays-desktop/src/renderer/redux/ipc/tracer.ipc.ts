import type { DataType } from '@/redux/ipc/index.ts';
import { invoke, ipcApi, processIpcResult } from '@/redux/ipc/index.ts';
import { saveLatestRender } from '@/indexed-db-image-cache.ts';

export const tracerIpcApi = ipcApi.injectEndpoints({
  endpoints: (builder) => ({
    render: builder.mutation<DataType<'tracer:TriggerRender'>, null>({
      queryFn: async () => {
        const result = await invoke('tracer:TriggerRender');
        return processIpcResult(result, (data) => data);
      },
    }),
    getRenderStatus: builder.query<DataType<'tracer:GetRenderStatus'>, null>({
      queryFn: async () => {
        const result = await invoke('tracer:GetRenderStatus');
        return processIpcResult(result, (data) => data);
      },
    }),
    loadRenderImage: builder.query<boolean, string>({
      queryFn: async (instanceUuid) => {
        const result = await invoke('tracer:GetRenderImageData');
        const { data: imageData } = processIpcResult(result, (data) => {
          return new Uint8Array(data);
        });

        await saveLatestRender(instanceUuid, imageData);
        return { data: true };
      },
    }),
    getTracerInstanceUuid: builder.query<
      DataType<'tracer:GetInstanceUuid'>,
      null
    >({
      queryFn: async () => {
        const result = await invoke('tracer:GetInstanceUuid');
        return processIpcResult(result, (data) => data);
      },
    }),
    getIntersectedUuidByPixelPos: builder.query<
      string | null,
      { x: number; y: number }
    >({
      queryFn: async ({ x, y }) => {
        const result = await invoke(
          'tracer:GetIntersectedUuidByPixelPos',
          x,
          y,
        );
        return processIpcResult(result, (data) => data);
      },
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
