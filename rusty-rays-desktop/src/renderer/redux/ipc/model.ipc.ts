import { ipcApi } from '@/redux/ipc/index.ts';
import type { Sphere } from 'rusty-rays-napi-node';

// todo: remove
/* eslint-disable @typescript-eslint/require-await */
async function getSpheres(): Promise<Sphere[]> {
  const sphere: Sphere = {
    uuid: 'foo',
    surface: 'surface',
    radius: 3.14,
    position: {
      x: 0,
      y: 0,
      z: 0,
    },
  };
  return [sphere];
}

export const modelsIpcApi = ipcApi.injectEndpoints({
  endpoints: (builder) => ({
    getAllSpheres: builder.query<Sphere[], null>({
      queryFn: async () => {
        const spheres = await getSpheres();
        return { data: spheres };
      },
    }),
  }),
  overrideExisting: false,
});

export const { useGetAllSpheresQuery } = modelsIpcApi;
