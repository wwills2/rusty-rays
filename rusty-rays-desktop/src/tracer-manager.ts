// tracer-manager.ts
import * as _ from 'lodash';
import type { RenderEvent } from 'rusty-rays-napi-node';
import type { RenderStatus } from '#/ipc/shared';
import { TracerSubprocessClient } from '#/tracer-subprocess-client';

const client = new TracerSubprocessClient('TODO');

type TracerInstance = { uuid: string } | undefined;

// Local mirror (same shape as before)
let tracerInstance: TracerInstance = undefined;
let tempRenderImageData: ArrayBuffer | undefined = undefined;

let renderStatus: RenderStatus = {
  renderProgressPercentage: undefined,
  writingImage: false,
  renderImageAvailable: false,
  renderErrorMsg: undefined,
  tracerInstanceUuid: undefined,
};

// Stream render events from child into local status
client.onRenderEvent((payload) => {
  const event = payload as
    | RenderEvent
    | { type: 'InternalError'; message: string };

  if (!event || typeof (event as any).type !== 'string') return;

  switch ((event as any).type) {
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
      // you may choose to clear state here; current code clears on fetch/take
      break;
    case 'Error':
      renderStatus.renderErrorMsg = 'Render failed: ' + event.message;
      break;
    case 'InternalError':
      renderStatus.renderErrorMsg = (event as any).message;
      break;
    default:
      break;
  }
});

function getTracerInstance(): TracerInstance {
  return tracerInstance;
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

async function setModelFromFilePath(path: string) {
  if (!_.isNil(renderStatus.renderProgressPercentage)) {
    throw new Error('Cannot set model. render in progress');
  }

  tempRenderImageData = undefined;

  const { instanceUuid } = await client.invoke<{ instanceUuid: string }>(
    'model.initFromFilePath',
    [path],
    120_000,
  );

  tracerInstance = { uuid: instanceUuid };
  resetRenderStatus();
  return instanceUuid;
}

async function setModelFromFileTextString(fileText: string) {
  if (!_.isNil(renderStatus.renderProgressPercentage)) {
    throw new Error('Cannot set model. render in progress');
  }

  tempRenderImageData = undefined;

  const { instanceUuid } = await client.invoke<{ instanceUuid: string }>(
    'model.initFromFileTextString',
    [fileText],
    120_000,
  );

  tracerInstance = { uuid: instanceUuid };
  resetRenderStatus();
  return instanceUuid;
}

async function setModel(undefinedModel: undefined) {
  if (!_.isNil(renderStatus.renderProgressPercentage)) {
    throw new Error('Cannot set model. render in progress');
  }
  tempRenderImageData = undefined;
  await client.invoke('model.setModel', [undefinedModel]);
  tracerInstance = undefined;
  resetRenderStatus();
}

async function triggerRender() {
  if (!_.isNil(renderStatus.renderProgressPercentage)) {
    throw new Error('Render already in progress');
  }

  resetRenderStatus();

  if (!tracerInstance) {
    renderStatus.renderErrorMsg =
      'No model loaded. A model must be loaded to render';
    return;
  }

  renderStatus.renderProgressPercentage = 0;
  await client.invoke('tracer.triggerRender', [], 5_000); // returns immediately; events stream
}

async function cancelRender() {
  await client.invoke('tracer.cancelRender', [], 10_000);
  resetRenderStatus();
}

async function takeRenderImageData() {
  resetRenderStatus();
  const imageData = await client.invoke<ArrayBuffer | undefined>(
    'tracer.takeRenderImageData',
    [],
    60_000,
  );
  tempRenderImageData = undefined;
  return imageData;
}

async function getIntersectedUuidByPixelPos(x: number, y: number) {
  if (!tracerInstance) {
    throw new Error(
      'No model loaded. A model must be loaded to query intersections',
    );
  }
  return await client.invoke<string | null>(
    'tracer.getIntersectedUuidByPixelPos',
    [x, y],
    30_000,
  );
}

export {
  // keep old exports (you’ll wire model init channels to the new setters)
  getTracerInstance,
  triggerRender,
  cancelRender,
  getRenderStatus,
  takeRenderImageData,

  // new setters you’ll call from your model IPC handlers
  setModelFromFilePath,
  setModelFromFileTextString,
  setModel,
  getIntersectedUuidByPixelPos,
};

export type { TracerInstance };
