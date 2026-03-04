import type { Cone, Polygon, Sphere, Triangle } from 'rusty-rays-napi-node';

const ModelChannels = {
  'model:InitFromFilePath': {
    args: [] as unknown as [path: string],
    dataType: {} as { instanceUuid: string },
  },
  'model:InitFromFileTextString': {
    args: [] as unknown as [fileContent: string],
    dataType: {} as { instanceUuid: string },
  },
  'model:getAllSpheres': {
    args: [],
    dataType: {} as Record<string, Sphere>,
  },
  'model:getAllCones': {
    args: [],
    dataType: {} as Record<string, Cone>,
  },
  'model:getAllTriangles': {
    args: [],
    dataType: {} as Record<string, Triangle>,
  },
  'model:getAllPolygons': {
    args: [],
    dataType: {} as Record<string, Polygon>,
  },
  'model:SetModel': {
    args: [] as unknown as [modelUuid: string | undefined],
    dataType: {} as boolean,
  },
};

export { ModelChannels };
