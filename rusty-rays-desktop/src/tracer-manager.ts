/**
 * utility for managing the reference to the currently open model.
 * the model has an internal semaphore.
 */

import { v4 as uuidv4 } from 'uuid';
import { Model, Tracer } from 'rusty-rays-napi-node';
import type { RenderStatus } from '#/ipc/shared';

type TracerInstance =
  | { uuid: string; model: Model; tracer: Tracer }
  | undefined;

let tracerInstance: TracerInstance = undefined;
let tempRenderImageData: ArrayBuffer | undefined = undefined;
let renderStatus: RenderStatus = {
  renderInProgress: false,
  renderImageAvailable: false,
  renderErrorMsg: undefined,
  tracerInstanceUuid: undefined,
};

function getTracerInstance(): TracerInstance | undefined {
  return tracerInstance;
}

function setModel(model: Model | undefined): string | undefined {
  if (renderStatus.renderInProgress) {
    throw new Error('Cannot set model. render in progress');
  }

  tempRenderImageData = undefined;

  if (model) {
    tracerInstance = { uuid: uuidv4(), model, tracer: new Tracer(model) };
    resetRenderStatus();
    return tracerInstance.uuid;
  } else {
    tracerInstance = undefined;
    resetRenderStatus();
  }
}

async function triggerRender() {
  if (renderStatus.renderInProgress) {
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
    renderStatus.renderInProgress = true;
    const imageData = await instance.tracer.renderToImageBuffer('png');
    tempRenderImageData = new Uint8Array(imageData).slice().buffer;
    renderStatus.renderImageAvailable = true;
  } catch (error) {
    if (error instanceof Error) {
      renderStatus.renderErrorMsg = error.message;
    } else {
      renderStatus.renderErrorMsg = `Unknown error: ${JSON.stringify(error)}`;
    }
  } finally {
    renderStatus.renderInProgress = false;
  }
}

function takeRenderImageData() {
  resetRenderStatus();
  const copy = tempRenderImageData;
  tempRenderImageData = undefined;
  return copy;
}

function getRenderStatus() {
  return { ...renderStatus, tracerInstanceUuid: tracerInstance?.uuid };
}

function resetRenderStatus() {
  if (renderStatus.renderInProgress) {
    throw new Error('Cannot reset render status. Render in progress');
  }

  renderStatus = {
    renderInProgress: false,
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
};
export type { TracerInstance };
