import type { RenderStatus } from './index';

const TracerChannels = {
  'tracer:TriggerRender': {
    args: [],
    dataType: {} as true,
  },
  'tracer:GetRenderStatus': {
    args: [],
    dataType: {} as RenderStatus,
  },
  'tracer:GetInstanceUuid': {
    args: [],
    dataType: {} as string,
  },
  'tracer:GetRenderImageData': {
    args: [],
    dataType: {} as ArrayBuffer,
  },
  'tracer:GetIntersectedUuidByPixelPos': {
    args: [] as unknown as [i: number, j: number],
    dataType: {} as string | null,
  },
};

export { TracerChannels };
