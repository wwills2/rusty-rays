// tracer-subprocess.ts
import { v4 as uuidv4 } from 'uuid';
import * as _ from 'lodash';
import type { RenderEvent } from 'rusty-rays-napi-node';
import { Model, Tracer } from 'rusty-rays-napi-node';
import type { RenderStatus } from '#/ipc/shared';

import type {
  RenderEventEnvelope,
  RpcRequest,
  RpcResponse,
  SubprocessArgs,
  SubprocessEvent,
  SubprocessResult
} from './sub-process-shared';
import { isRpcRequest, serializeError } from './sub-process-shared';

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

function send(msg: RpcResponse | SubprocessEvent): void {
  if (typeof process.send === 'function') process.send(msg);
}

function emitRenderEvent(payload: RenderEventEnvelope): void {
  send({ kind: 'event', event: 'render:Event', payload });
}

function resetRenderStatus(): void {
  if (!_.isNil(renderStatus.renderProgressPercentage)) {
    throw new Error('Cannot reset render status. Render in progress');
  }

  renderStatus = {
    tracerInstanceUuid: tracerInstance?.uuid,
    renderProgressPercentage: undefined,
    writingImage: false,
    renderErrorMsg: undefined,
    renderImageAvailable: false,
  };
}

async function setModel(model: Model | undefined): Promise<string | undefined> {
  if (!_.isNil(renderStatus.renderProgressPercentage)) {
    throw new Error('Cannot set model. render in progress');
  }

  tempRenderImageData = undefined;

  if (model !== undefined) {
    tracerInstance = {
      uuid: uuidv4(),
      model,
      tracer: await Tracer.create(model),
    };
    resetRenderStatus();
    return tracerInstance.uuid;
  }

  tracerInstance = undefined;
  resetRenderStatus();
  return undefined;
}

async function triggerRender(): Promise<void> {
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

    const onRenderEvent = (error: unknown, event: RenderEvent): void => {
      if (error && !renderStatus.renderErrorMsg) {
        renderStatus.renderErrorMsg =
          'Received error during render: ' + JSON.stringify(error);

        emitRenderEvent({
          type: 'InternalError',
          message: renderStatus.renderErrorMsg,
        });

        void instance.tracer.cancelRender().catch(() => {});
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

      emitRenderEvent({ type: 'RenderEvent', event });
    };

    const imageData = await instance.tracer.renderToImageBuffer(
      'png',
      50,
      onRenderEvent,
    );

    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    if (!canceled) {
      tempRenderImageData = new Uint8Array(imageData).slice().buffer;
      renderStatus.renderImageAvailable = true;
    } else {
      resetRenderStatus();
      tempRenderImageData = undefined;
    }
  } catch (err) {
    const msg =
      err instanceof Error
        ? err.message
        : `Unknown error: ${JSON.stringify(err)}`;
    renderStatus.renderErrorMsg = msg;
    emitRenderEvent({ type: 'InternalError', message: msg });
  } finally {
    renderStatus.renderProgressPercentage = undefined;
  }
}

async function cancelRender(): Promise<void> {
  if (tracerInstance) {
    await tracerInstance.tracer.cancelRender();
  }
  resetRenderStatus();
}

function getRenderStatus(): RenderStatus {
  return { ...renderStatus, tracerInstanceUuid: tracerInstance?.uuid };
}

function takeRenderImageData(): ArrayBuffer | undefined {
  resetRenderStatus();
  const copy = tempRenderImageData;
  tempRenderImageData = undefined;
  return copy;
}

async function handleRpc(
  req: RpcRequest,
): Promise<SubprocessResult<typeof req.method>> {
  switch (req.method) {
    case 'health:Ping': {
      const result: SubprocessResult<'health:Ping'> = {
        ok: true,
        pid: process.pid,
      };
      return result;
    }

    case 'model:InitFromFilePath': {
      const [path] = req.args as SubprocessArgs<'model:InitFromFilePath'>;
      const model = await Model.fromFilePath(path);
      const instanceUuid = await setModel(model);
      if (!instanceUuid)
        throw new Error(
          `invalid tracer instance uuid. received ${instanceUuid}`,
        );
      return {
        instanceUuid,
      };
    }

    case 'model:InitFromFileTextString': {
      const [fileText] =
        req.args as SubprocessArgs<'model:InitFromFileTextString'>;
      const model = Model.fromString(fileText);
      const instanceUuid = await setModel(model);
      if (!instanceUuid)
        throw new Error(
          `invalid tracer instance uuid. received ${instanceUuid}`,
        );
      return {
        instanceUuid,
      };
    }

    case 'model:SetModel': {
      const [uuid] = req.args as SubprocessArgs<'model:SetModel'>;
      if (uuid === undefined) {
        await setModel(undefined);
        const result: SubprocessResult<'model:SetModel'> = true;
        return result as SubprocessResult<typeof req.method>;
      }
      throw new Error(`cannot load model with uuid ${uuid}`);
    }

    case 'model:GetAllSpheres': {
      if (!tracerInstance)
        throw new Error('failed to fetch spheres. no model loaded');
      return await tracerInstance.model.allSpheres;
    }

    case 'model:GetAllCones': {
      if (!tracerInstance)
        throw new Error('failed to fetch cones. no model loaded');
      return await tracerInstance.model.allCones;
    }

    case 'model:GetAllTriangles': {
      if (!tracerInstance)
        throw new Error('failed to fetch triangles. no model loaded');
      return await tracerInstance.model.allTriangles;
    }

    case 'model:GetAllPolygons': {
      if (!tracerInstance)
        throw new Error('failed to fetch polygons. no model loaded');
      return await tracerInstance.model.allPolygons;
    }

    case 'tracer:GetInstanceUuid': {
      return tracerInstance
        ? tracerInstance.uuid
        : 'TRACER_INSTANCE_NOT_LOADED';
    }

    case 'tracer:TriggerRender': {
      // eslint wants explicit void when not awaiting
      void triggerRender().catch((e: unknown) => {
        const msg = serializeError(e).message;
        emitRenderEvent({ type: 'InternalError', message: msg });
      });
      const result: SubprocessResult<'tracer:TriggerRender'> = true;
      return result as SubprocessResult<typeof req.method>;
    }

    case 'tracer:CancelRender': {
      await cancelRender();
      const result: SubprocessResult<'tracer:CancelRender'> = true;
      return result as SubprocessResult<typeof req.method>;
    }

    case 'tracer:GetRenderStatus': {
      return getRenderStatus();
    }

    case 'tracer:TakeRenderImageData': {
      return takeRenderImageData();
    }

    case 'tracer:GetIntersectedUuidByPixelPos': {
      if (!tracerInstance) {
        throw new Error(
          'No model loaded. A model must be loaded to query intersections',
        );
      }
      const [x, y] =
        req.args as SubprocessArgs<'tracer:GetIntersectedUuidByPixelPos'>;
      return await tracerInstance.tracer.getIntersectedUuidByPixelPos(x, y);
    }

    default: {
      const _never: never = req.method;
      throw new Error(`Unknown method: ${String(_never)}`);
    }
  }
}

/**
 * Fix @typescript-eslint/no-misused-promises:
 * don't make the event handler itself async; invoke an async IIFE with `void`.
 */
process.on('message', (msg: unknown) => {
  if (!isRpcRequest(msg)) return;

  void (async () => {
    try {
      const result = await handleRpc(msg);
      const resp: RpcResponse = {
        kind: 'rpc.response',
        id: msg.id,
        ok: true,
        result: result as never,
      };
      send(resp);
    } catch (err) {
      const resp: RpcResponse = {
        kind: 'rpc.response',
        id: msg.id,
        ok: false,
        error: serializeError(err),
      };
      send(resp);
    }
  })();
});

send({ kind: 'event', event: 'process:Ready', payload: { pid: process.pid } });
