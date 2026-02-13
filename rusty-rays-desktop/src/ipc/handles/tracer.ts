import { handle } from '#/ipc/handles/index';
import {
  getRenderStatus,
  getTracerInstance,
  takeRenderImageData,
  triggerRender,
} from '#/tracer-manager';
import { toIpcError } from '#/ipc/shared';

function initTracerChannels() {
  handle('tracer:GetInstanceUuid', () => {
    const instance = getTracerInstance();
    if (instance === undefined) {
      return { data: 'TRACER_INSTANCE_NOT_LOADED' };
    }
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
      // do not await, let this process continue in the background
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

  handle('tracer:GetRenderImageData', () => {
    const { renderErrorMsg } = getRenderStatus();
    if (renderErrorMsg) {
      const renderError = new Error(renderErrorMsg);
      return toIpcError(renderError, 'Render failed');
    }
    const renderImageData = takeRenderImageData();
    return { data: renderImageData };
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
      const { tracer } = instance;
      const maybeIntersectedObjectUuid =
        await tracer.getIntersectedUuidByPixelPos(x, y);
      return { data: maybeIntersectedObjectUuid };
    } catch (error: unknown) {
      return toIpcError(
        error,
        'Failed to get intersected uuid by pixel position',
      );
    }
  });
}

export { initTracerChannels };
