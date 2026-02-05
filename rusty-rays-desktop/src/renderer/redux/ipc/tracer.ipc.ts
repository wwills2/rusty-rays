import { invoke, ipcApi, processIpcResult } from '@/redux/ipc/index.ts';

export const modelsIpcApi = ipcApi.injectEndpoints({
  endpoints: (builder) => ({
    render: builder.query<Uint8Array<ArrayBuffer>, null>({
      queryFn: async () => {
        const result = await invoke('tracer:Render');
        return processIpcResult(result, (data) => {
          return new Uint8Array(data);
        });
      },
    }),
    getIntersectedUuidByPixelPos: builder.mutation<
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

export const { useRenderQuery, useGetIntersectedUuidByPixelPosMutation } =
  modelsIpcApi;
