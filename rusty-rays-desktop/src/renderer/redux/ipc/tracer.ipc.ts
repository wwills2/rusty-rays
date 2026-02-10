import type { DataType } from '@/redux/ipc/index.ts';
import { invoke, ipcApi, processIpcResult } from '@/redux/ipc/index.ts';
import {
  loadLatestRender,
  saveLatestRender,
} from '@/indexed-db-image-cache.ts';

export const tracerIpcApi = ipcApi.injectEndpoints({
  endpoints: (builder) => ({
    render: builder.mutation<DataType<'tracer:TriggerRender'>, null>({
      queryFn: async () => {
        const result = await invoke('tracer:TriggerRender');
        return processIpcResult(result, (data) => data);
      },
    }),
    isRenderInProgress: builder.query<
      DataType<'tracer:GetRenderProgress'>,
      null
    >({
      queryFn: async () => {
        const result = await invoke('tracer:GetRenderProgress');
        return processIpcResult(result, (data) => data);
      },
    }),
    loadRenderImage: builder.query<boolean, string>({
      queryFn: async (instanceUuid) => {
        // If already cached, do not fetch, do not touch Redux with bytes
        const cached = await loadLatestRender(instanceUuid);
        if (cached?.byteLength) {
          return { data: true };
        }

        // Fetch via IPC (if invoke or processIpcResult throws, we let it throw)
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
  useIsRenderInProgressQuery,
  useGetTracerInstanceUuidQuery,
  useLazyLoadRenderImageQuery,
} = tracerIpcApi;
