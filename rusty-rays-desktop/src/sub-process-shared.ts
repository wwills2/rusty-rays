// sub-process-shared.ts
import type {
  Cone,
  IntersectedObjectInfo,
  Polygon,
  RenderEvent,
  Sphere,
  Triangle,
} from 'rusty-rays-napi-node';
import type { RenderStatus } from '#/ipc/shared';

/**
 * RPC schema: method name -> args + result types.
 * Mirrors your Electron IPC typing pattern.
 */
const SubprocessRpc = {
  'health:Ping': {
    args: [],
    result: {} as { ok: true; pid: number },
  },

  // Model lifecycle
  'model:InitFromFilePath': {
    args: [] as unknown as [path: string],
    result: {} as { instanceUuid: string },
  },
  'model:InitFromFileTextString': {
    args: [] as unknown as [fileText: string],
    result: {} as { instanceUuid: string },
  },
  'model:SetModel': {
    args: [] as unknown as [modelUuid: string | undefined],
    result: {} as boolean,
  },

  // Model queries
  'model:GetAllSpheres': {
    args: [],
    result: {} as Record<string, Sphere>,
  },
  'model:GetAllCones': {
    args: [],
    result: {} as Record<string, Cone>,
  },
  'model:GetAllTriangles': {
    args: [],
    result: {} as Record<string, Triangle>,
  },
  'model:GetAllPolygons': {
    args: [],
    result: {} as Record<string, Polygon>,
  },

  // Tracer + render
  'tracer:GetInstanceUuid': {
    args: [],
    result: {} as string, // or "TRACER_INSTANCE_NOT_LOADED"
  },
  'tracer:TriggerRender': {
    args: [] as const,
    result: {} as true,
  },
  'tracer:CancelRender': {
    args: [] as const,
    result: {} as true,
  },
  'tracer:GetRenderStatus': {
    args: [] as const,
    result: {} as RenderStatus,
  },
  'tracer:TakeRenderImageData': {
    args: [] as const,
    result: {} as Buffer | undefined,
  },
  'tracer:GetIntersectedUuidByPixelPos': {
    args: [] as unknown as [x: number, y: number],
    result: {} as IntersectedObjectInfo | null, // <-- was string | null
  },
} as const;

type SubprocessRpcSchema = typeof SubprocessRpc;

type SubprocessMethod = keyof SubprocessRpcSchema;
type SubprocessArgs<M extends SubprocessMethod> =
  SubprocessRpcSchema[M]['args'];
type SubprocessResult<M extends SubprocessMethod> =
  SubprocessRpcSchema[M]['result'];

type SerializedError = {
  name: string;
  message: string;
  stack?: string;
  code?: string;
};

type RpcRequest<M extends SubprocessMethod = SubprocessMethod> = {
  kind: 'rpc.request';
  id: string;
  method: M;
  args: SubprocessArgs<M>;
};

type RpcResponse<M extends SubprocessMethod = SubprocessMethod> =
  | { kind: 'rpc.response'; id: string; ok: true; result: SubprocessResult<M> }
  | { kind: 'rpc.response'; id: string; ok: false; error: SerializedError };

type RenderEventEnvelope =
  | { type: 'RenderEvent'; event: RenderEvent }
  | { type: 'InternalError'; message: string };

type SubprocessEvent =
  | { kind: 'event'; event: 'process:Ready'; payload: { pid: number } }
  | { kind: 'event'; event: 'render:Event'; payload: RenderEventEnvelope };

type SubprocessMessage = RpcRequest | RpcResponse | SubprocessEvent;

function serializeError(err: unknown): SerializedError {
  if (err instanceof Error) {
    const maybeCode = (err as { code?: unknown }).code;
    return {
      name: err.name,
      message: err.message,
      stack: err.stack,
      code: typeof maybeCode === 'string' ? maybeCode : undefined,
    };
  }

  return {
    name: 'Error',
    message: typeof err === 'string' ? err : JSON.stringify(err),
  };
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function isRpcRequest(value: unknown): value is RpcRequest {
  if (!isObject(value)) return false;
  return value['kind'] === 'rpc.request' && typeof value['id'] === 'string';
}

function isRpcResponse(value: unknown): value is RpcResponse {
  if (!isObject(value)) return false;
  return value['kind'] === 'rpc.response' && typeof value['id'] === 'string';
}

function isSubprocessEvent(value: unknown): value is SubprocessEvent {
  if (!isObject(value)) return false;
  return value['kind'] === 'event' && typeof value['event'] === 'string';
}

export {
  SubprocessRpc,
  serializeError,
  isRpcRequest,
  isRpcResponse,
  isSubprocessEvent,
};
export type {
  SubprocessRpcSchema,
  SubprocessMethod,
  SubprocessArgs,
  SubprocessResult,
  SerializedError,
  RpcRequest,
  RpcResponse,
  RenderEventEnvelope,
  SubprocessEvent,
  SubprocessMessage,
};
