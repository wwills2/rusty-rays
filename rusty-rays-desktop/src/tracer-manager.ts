/**
 * utility for managing the reference to the currently open model.
 * the model has an internal semaphore.
 */

import { v4 as uuidv4 } from 'uuid';
import { Model, Tracer } from 'rusty-rays-napi-node';

type TracerInstance =
  | { uuid: string; model: Model; tracer: Tracer }
  | undefined;

let tracerInstance: TracerInstance = undefined;
let tempRenderImageData: ArrayBuffer | undefined = undefined;
let renderInProgress = false;
let renderError: Error | undefined = undefined;

function getTracerInstance(): TracerInstance | undefined {
  return tracerInstance;
}

function setModel(model: Model | undefined): string | undefined {
  if (renderInProgress) {
    throw new Error('Cannot set model. render in progress');
  }

  tempRenderImageData = undefined;
  renderError = undefined;

  if (model) {
    tracerInstance = { uuid: uuidv4(), model, tracer: new Tracer(model) };
    return tracerInstance.uuid;
  } else {
    tracerInstance = undefined;
  }
}

async function triggerRender() {
  renderError = undefined;

  if (!tracerInstance) {
    renderError = new Error(
      'No model loaded. A model must be loaded to render',
    );
    return;
  }

  if (renderInProgress) {
    renderError = new Error('Render already in progress');
  }

  try {
    renderInProgress = true;
    const imageData = await tracerInstance.tracer.renderToImageBuffer('png');
    tempRenderImageData = new Uint8Array(imageData).slice().buffer;
    renderError = undefined;
  } catch (error) {
    if (error instanceof Error) {
      renderError = error;
    } else {
      renderError = new Error(`Unknown error: ${JSON.stringify(error)}`);
    }
  } finally {
    renderInProgress = false;
  }
}

function isRenderInProgress() {
  return renderInProgress;
}

function isRenderImageAvialable() {
  return !!tempRenderImageData;
}

function takeRenderImageData() {
  const copy = tempRenderImageData;
  tempRenderImageData = undefined;
  return copy;
}

function takeRenderError() {
  const copy = renderError;
  renderError = undefined;
  return copy;
}

export {
  getTracerInstance,
  setModel,
  triggerRender,
  takeRenderImageData,
  takeRenderError,
  isRenderInProgress,
  isRenderImageAvialable,
};
export type { TracerInstance };
