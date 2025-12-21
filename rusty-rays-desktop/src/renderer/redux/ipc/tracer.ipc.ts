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
  }),
  overrideExisting: false,
});

export const { useRenderQuery } = modelsIpcApi;
