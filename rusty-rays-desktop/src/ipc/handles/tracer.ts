import { handle } from '#/ipc/handles/index';
import {
  getTracerInstance,
  isRenderImageAvialable,
  isRenderInProgress,
  takeRenderError,
  takeRenderImageData,
  triggerRender,
} from '#/tracer-manager';
import { toIpcError } from '#/ipc/shared';

function initTracerChannels() {
  handle('tracer:GetInstanceUuid', () => {
    const instance = getTracerInstance();
    if (instance === undefined) {
      return { data: undefined };
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
      triggerRender().catch(() => {});
      return { data: true };
    } catch (error: unknown) {
      return toIpcError(error, 'Failed to start render.');
    }
  });

  handle('tracer:GetRenderProgress', () => {
    const instance = getTracerInstance();
    if (instance === undefined) {
      return { data: false };
    }
    const renderInProgress = isRenderInProgress();
    return { data: renderInProgress };
  });

  handle('tracer:GetIsRenderAvailable', () => {
    const renderAvailable = isRenderImageAvialable();
    return { data: renderAvailable };
  });

  handle('tracer:GetRenderImageData', () => {
    const renderError = takeRenderError();
    if (renderError) {
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
