import type { Sphere } from 'rusty-rays-napi-node';

const ModelChannels = {
  'model:InitFromFilePath': {
    args: [] as unknown as [path: string],
    dataType: {} as true,
  },
  'model:InitFromFileTextString': {
    args: [] as unknown as [fileContent: string],
    dataType: {} as true,
  },
  'model:getAllSpheres': {
    args: [],
    dataType: {} as Record<string, Sphere>,
  },
};

export { ModelChannels };
