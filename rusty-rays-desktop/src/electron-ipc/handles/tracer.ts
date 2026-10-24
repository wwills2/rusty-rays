// src/electron-ipc/handles/tracer.ts
import { handle } from '#/electron-ipc/handles/index';
import {
  getIntersectedUuidByPixelPos,
  getRenderStatus,
  getTracerInstance,
  takeRenderImageData,
  triggerRender,
} from '#/tracer-manager';

function initTracerChannels() {
  handle('tracer:GetInstanceUuid', () => {
    const instance = getTracerInstance();
    if (instance === undefined) return { data: 'TRACER_INSTANCE_NOT_LOADED' };
    return { data: instance.uuid };
  });

  handle('tracer:TriggerRender', () => {
    const instance = getTracerInstance();
    if (instance === undefined) {
      return {
        error: new Error('No model loaded. A model must be loaded to render'),
      };
    }

    // fire-and-forget; status updates and errors come from streamed events
    triggerRender().catch(() => {});
    return { data: true };
  });

  handle('tracer:GetRenderStatus', () => {
    const data = getRenderStatus();
    return { data };
  });

  handle('tracer:GetRenderImageData', async () => {
    const data = await takeRenderImageData();
    return { data };
  });

  handle('tracer:GetIntersectedUuidByPixelPos', async (_, args) => {
    const instance = getTracerInstance();
    if (instance === undefined) {
      return {
        error: new Error(
          'No model loaded. A model must be loaded to query intersections',
        ),
      };
    }

    const [x, y] = args;
    const data = await getIntersectedUuidByPixelPos(x, y);
    return { data };
  });
}

export { initTracerChannels };
