// tracer-subprocess-shared.ts
import type {
  Cone,
  IntersectedObjectInfo,
  Polygon,
  RenderEvent,
  Sphere,
  Triangle,
} from 'rusty-rays-napi-node';
import type { RenderStatus } from '#/electron-ipc/shared';

/**
 * RPC schema: method name -> args + result types.
 * Mirrors your Electron IPC typing pattern.
 */
const SubprocessRpc = {
  'subProcIpc:Health:Ping': {
    args: [],
    result: {} as { ok: true; pid: number },
  },

  // Model lifecycle
  'subProcIpc:Model:InitFromFilePath': {
    args: [] as unknown as [path: string],
    result: {} as { instanceUuid: string },
  },
  'subProcIpc:Model:InitFromFileTextString': {
    args: [] as unknown as [fileText: string],
    result: {} as { instanceUuid: string },
  },
  'subProcIpc:Model:SetModel': {
    args: [] as unknown as [modelUuid: string | undefined],
    result: {} as boolean,
  },

  // Model queries
  'subProcIpc:Model:GetAllSpheres': {
    args: [],
    result: {} as Record<string, Sphere>,
  },
  'subProcIpc:Model:GetAllCones': {
    args: [],
    result: {} as Record<string, Cone>,
  },
  'subProcIpc:Model:GetAllTriangles': {
    args: [],
    result: {} as Record<string, Triangle>,
  },
  'subProcIpc:Model:GetAllPolygons': {
    args: [],
    result: {} as Record<string, Polygon>,
  },

  // Tracer + render
  'subProcIpc:Tracer:GetInstanceUuid': {
    args: [],
    result: {} as string, // or "TRACER_INSTANCE_NOT_LOADED"
  },
  'subProcIpc:Tracer:TriggerRender': {
    args: [] as const,
    result: {} as true,
  },
  'subProcIpc:Tracer:CancelRender': {
    args: [] as const,
    result: {} as true,
  },
  'subProcIpc:Tracer:GetRenderStatus': {
    args: [] as const,
    result: {} as RenderStatus,
  },
  'subProcIpc:Tracer:TakeRenderImageData': {
    args: [] as const,
    result: {} as Buffer | undefined,
  },
  'subProcIpc:Tracer:GetIntersectedUuidByPixelPos': {
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
