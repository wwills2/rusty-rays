import type { Sphere } from 'rusty-rays-napi-node';

const ModelChannels = {
  'model:InitFromFile': {
    args: [] as unknown as [path: string],
    dataType: {} as true,
  },
  'model:getAllSpheres': {
    args: [],
    dataType: [] as Sphere[],
  },
};

export { ModelChannels };
