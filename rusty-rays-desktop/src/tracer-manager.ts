// tracer-manager.ts
import * as _ from 'lodash';
import type { RenderEvent } from 'rusty-rays-napi-node';
import type { RenderStatus } from '#/ipc/shared';
import { TracerSubprocessClient } from '#/tracer-subprocess-client';

const client = new TracerSubprocessClient('TODO');

type TracerInstance = { uuid: string } | undefined;

// Local mirror
let tracerInstance: TracerInstance = undefined;

let renderStatus: RenderStatus = {
  renderProgressPercentage: undefined,
  writingImage: false,
  renderImageAvailable: false,
  renderErrorMsg: undefined,
  tracerInstanceUuid: undefined,
};

// Stream render events from child into local status
client.onRenderEvent((payload) => {
  // payload is RenderEventEnvelope; narrow it first
  if (payload.type === 'InternalError') {
    renderStatus.renderErrorMsg = payload.message;
    return;
  }

  const event: RenderEvent = payload.event;

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
      // optional: mark image available when Finished arrives (or keep it purely server-driven)
      // renderStatus.renderImageAvailable = true;
      break;
    case 'Canceled':
      // optional: clear status or leave to TakeRenderImageData/reset
      break;
    case 'Error':
      renderStatus.renderErrorMsg = 'Render failed: ' + event.message;
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

  const { instanceUuid } = await client.invoke(
    'model:InitFromFilePath',
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

  const { instanceUuid } = await client.invoke(
    'model:InitFromFileTextString',
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

  await client.invoke('model:SetModel', [undefinedModel]);
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
  await client.invoke('tracer:TriggerRender', [], 5_000);
}

async function cancelRender() {
  await client.invoke('tracer:CancelRender', [], 10_000);
  resetRenderStatus();
}

async function takeRenderImageData() {
  resetRenderStatus();
  const imageData = await client.invoke(
    'tracer:TakeRenderImageData',
    [],
    60_000,
  );
  return imageData;
}

async function getIntersectedUuidByPixelPos(x: number, y: number) {
  if (!tracerInstance) {
    throw new Error(
      'No model loaded. A model must be loaded to query intersections',
    );
  }

  // NOTE: return type updated below (IntersectedObjectInfo | null)
  return await client.invoke(
    'tracer:GetIntersectedUuidByPixelPos',
    [x, y],
    30_000,
  );
}

async function getAllSpheres() {
  if (!tracerInstance)
    throw new Error('failed to fetch spheres. no model loaded');
  return await client.invoke('model:GetAllSpheres', [], 30_000);
}
async function getAllCones() {
  if (!tracerInstance)
    throw new Error('failed to fetch cones. no model loaded');
  return await client.invoke('model:GetAllCones', [], 30_000);
}
async function getAllTriangles() {
  if (!tracerInstance)
    throw new Error('failed to fetch triangles. no model loaded');
  return await client.invoke('model:GetAllTriangles', [], 30_000);
}
async function getAllPolygons() {
  if (!tracerInstance)
    throw new Error('failed to fetch polygons. no model loaded');
  return await client.invoke('model:GetAllPolygons', [], 30_000);
}

export {
  getTracerInstance,
  triggerRender,
  cancelRender,
  getRenderStatus,
  takeRenderImageData,
  setModelFromFilePath,
  setModelFromFileTextString,
  setModel,
  getIntersectedUuidByPixelPos,
  getAllSpheres,
  getAllCones,
  getAllTriangles,
  getAllPolygons,
};

export type { TracerInstance };
