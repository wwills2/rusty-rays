// src/electron-ipc/handles/tracer.ts
import { handle } from '#/electron-ipc/handles/index';
import {
  getIntersectedUuidByPixelPos,
  getRenderStatus,
  getTracerInstance,
  takeRenderImageData,
  triggerRender,
} from '#/tracer-manager';
import { toIpcError } from '#/electron-ipc/shared';

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

    try {
      // fire-and-forget; status updates and errors come from streamed events
      triggerRender().catch(() => {});
      return { data: true };
    } catch (error: unknown) {
      return toIpcError(error, 'Failed to start render.');
    }
  });

  handle('tracer:GetRenderStatus', () => {
    const data = getRenderStatus();
    return { data };
  });

  handle('tracer:GetRenderImageData', async () => {
    try {
      const data = await takeRenderImageData();
      return { data };
    } catch (error) {
      return toIpcError(error, 'Render failed');
    }
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

    try {
      const [x, y] = args;
      const data = await getIntersectedUuidByPixelPos(x, y);
      return { data };
    } catch (error: unknown) {
      return toIpcError(
        error,
        'Failed to get intersected uuid by pixel position',
      );
    }
  });
}

export { initTracerChannels };
