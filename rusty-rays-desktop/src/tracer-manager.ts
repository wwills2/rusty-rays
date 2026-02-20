/**
 * utility for managing the reference to the currently open model.
 * the model has an internal semaphore.
 */

import { v4 as uuidv4 } from 'uuid';
import type { RenderEvent } from 'rusty-rays-napi-node';
import { Model, Tracer } from 'rusty-rays-napi-node';
import type { RenderStatus } from '#/ipc/shared';
import * as _ from 'lodash';

type TracerInstance =
  | { uuid: string; model: Model; tracer: Tracer }
  | undefined;

let tracerInstance: TracerInstance = undefined;
let tempRenderImageData: ArrayBuffer | undefined = undefined;
let renderStatus: RenderStatus = {
  renderProgressPercentage: undefined,
  writingImage: false,
  renderImageAvailable: false,
  renderErrorMsg: undefined,
  tracerInstanceUuid: undefined,
};

function getTracerInstance(): TracerInstance | undefined {
  return tracerInstance;
}

async function setModel(model: Model | undefined) {
  if (!_.isNil(renderStatus.renderProgressPercentage)) {
    throw new Error('Cannot set model. render in progress');
  }

  tempRenderImageData = undefined;

  if (model) {
    tracerInstance = {
      uuid: uuidv4(),
      model,
      tracer: await Tracer.create(model),
    };
    resetRenderStatus();
    return tracerInstance.uuid;
  } else {
    tracerInstance = undefined;
    resetRenderStatus();
  }
}

async function triggerRender() {
  if (!_.isNil(renderStatus.renderProgressPercentage)) {
    throw new Error('Render already in progress');
  }

  resetRenderStatus();

  const instance = tracerInstance;
  if (!instance) {
    renderStatus.renderErrorMsg =
      'No model loaded. A model must be loaded to render';
    return;
  }

  try {
    let canceled = false;
    renderStatus.renderProgressPercentage = 0;
    const onRenderEvent = (error: unknown, event: RenderEvent) => {
      if (error && !renderStatus.renderErrorMsg) {
        renderStatus.renderErrorMsg =
          'Received error during render: ' + JSON.stringify(error);
        tracerInstance?.tracer.cancelRender().catch((error: unknown) => {
          console.error(
            'an error occurred while trying to cancel render:',
            error,
          );
        });
        return;
      }

      switch (event.type) {
        case 'Progress':
          renderStatus.renderProgressPercentage = event.percent;
          break;
        case 'WritingImage':
          renderStatus.writingImage = true;
          break;
        case 'Finished':
          renderStatus.renderProgressPercentage = undefined;
          renderStatus.writingImage = false;
          break;
        case 'Canceled':
          canceled = true;
          break;
        case 'Error':
          renderStatus.renderErrorMsg = 'Render failed: ' + event.message;
          break;
        default:
          break;
      }
    };

    const imageData = await instance.tracer.renderToImageBuffer(
      'png',
      50,
      onRenderEvent,
    );

    if (!canceled) {
      tempRenderImageData = new Uint8Array(imageData).slice().buffer;
      renderStatus.renderImageAvailable = true;
    } else {
      resetRenderStatus();
      tempRenderImageData = undefined;
    }
  } catch (error) {
    if (error instanceof Error) {
      renderStatus.renderErrorMsg = error.message;
    } else {
      renderStatus.renderErrorMsg = `Unknown error: ${JSON.stringify(error)}`;
    }
  } finally {
    renderStatus.renderProgressPercentage = undefined;
  }
}

function takeRenderImageData() {
  resetRenderStatus();
  const copy = tempRenderImageData;
  tempRenderImageData = undefined;
  return copy;
}

async function cancelRender() {
  try {
    await tracerInstance?.tracer.cancelRender();
    resetRenderStatus();
  } catch (error) {
    if (error instanceof Error) {
      console.error(
        'an error occurred while trying to cancel render:',
        error.message,
      );
    } else {
      console.error(
        'an unknown error occurred while trying to cancel render:',
        error,
      );
    }
  }
}

function getRenderStatus() {
  return { ...renderStatus, tracerInstanceUuid: tracerInstance?.uuid };
}

function resetRenderStatus() {
  if (!_.isNil(renderStatus.renderProgressPercentage)) {
    throw new Error('Cannot reset render status. Render in progress');
  }

  renderStatus = {
    renderProgressPercentage: undefined,
    writingImage: false,
    renderImageAvailable: false,
    renderErrorMsg: undefined,
    tracerInstanceUuid: tracerInstance?.uuid,
  };
}

export {
  getTracerInstance,
  setModel,
  triggerRender,
  takeRenderImageData,
  getRenderStatus,
  cancelRender,
};
export type { TracerInstance };
