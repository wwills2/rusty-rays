import type { Sphere } from 'rusty-rays-napi-node';

export type ModelChannels = {
  'model:InitFromFile': {
    args: [path: string];
    result: true;
  };
  'model:getAllSpheres': {
    args: null;
    result: Sphere[];
  };
};
