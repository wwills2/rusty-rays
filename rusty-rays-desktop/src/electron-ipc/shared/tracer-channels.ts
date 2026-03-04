import type { RenderStatus } from './index';
import type { IntersectedObjectInfo } from 'rusty-rays-napi-node';

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
    dataType: {} as Buffer,
  },
  'tracer:GetIntersectedUuidByPixelPos': {
    args: [] as unknown as [x: number, y: number],
    dataType: {} as IntersectedObjectInfo | null,
  },
};

export { TracerChannels };
