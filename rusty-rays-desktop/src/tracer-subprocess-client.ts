// tracer-subprocess-client.ts
import { type ChildProcess, fork } from 'node:child_process';
import { v4 as uuidv4 } from 'uuid';

import type {
  RenderEventEnvelope,
  RpcRequest,
  SubprocessArgs,
  SubprocessEvent,
  SubprocessMethod,
  SubprocessResult,
} from './sub-process-shared';
import { isRpcResponse, isSubprocessEvent } from './sub-process-shared';
import * as inspector from 'node:inspector';
import { app } from 'electron';
import * as path from 'node:path';

type PendingCall = {
  resolve: (value: unknown) => void;
  reject: (err: Error) => void;
  timeout: NodeJS.Timeout;
};

export class TracerSubprocessClient {
  private child: ChildProcess | null = null;
  private pending = new Map<string, PendingCall>();
  private renderHandlers = new Set<(payload: RenderEventEnvelope) => void>();
  private started = false;
  private readonly entryPath: string;
  private readonly debuggerAttached = inspector.url() !== undefined;

  constructor() {
    this.entryPath = this.getSubprocessNodeSourcePath();
  }

  onRenderEvent(handler: (payload: RenderEventEnvelope) => void): () => void {
    this.renderHandlers.add(handler);
    return () => this.renderHandlers.delete(handler);
  }

  async start(): Promise<void> {
    if (this.started) return;
    this.started = true;

    const childExecPath = this.getSubprocessNodeExecPath();
    console.log(
      'executing tracer subprocess with node executable: ',
      childExecPath,
    );

    const child = fork(this.entryPath, [], {
      execPath: childExecPath,
      stdio: ['inherit', 'inherit', 'inherit', 'ipc'],
      env: process.env,
      serialization: 'advanced',
    });

    this.child = child;

    child.on('message', (msg: unknown) => {
      this.handleMessage(msg);
    });
    child.on('exit', (code, signal) => {
      const reason = new Error(
        `tracer subprocess exited (code=${String(code)}, signal=${String(signal)})`,
      );

      for (const [id, p] of this.pending.entries()) {
        clearTimeout(p.timeout);
        p.reject(reason);
        this.pending.delete(id);
      }

      this.child = null;
      this.started = false;
    });

    await this.invoke('health:Ping', [], 10_000);
  }

  stop(): void {
    this.child?.kill();
    this.child = null;
    this.started = false;
  }

  async invoke<M extends SubprocessMethod>(
    method: M,
    args: SubprocessArgs<M>,
    timeoutMs = 30_000,
  ): Promise<SubprocessResult<M>> {
    if (!this.child) {
      await this.start();
    }

    const child = this.child;
    if (!child || typeof child.send !== 'function') {
      throw new Error('tracer subprocess not available');
    }

    const id = uuidv4();
    const req: RpcRequest<M> = { kind: 'rpc.request', id, method, args };

    const result = await new Promise<unknown>((resolve, reject) => {
      const timeout = setTimeout(() => {
        if (!this.debuggerAttached) {
          this.pending.delete(id);
          reject(new Error(`RPC timeout (${timeoutMs}ms): ${method}`));
        }
      }, timeoutMs);

      const rejectAsError = (e: unknown) => {
        if (e instanceof Error) reject(e);
        else reject(new Error(typeof e === 'string' ? e : JSON.stringify(e)));
      };

      this.pending.set(id, { resolve, reject: rejectAsError, timeout });
      child.send(req);
    });

    return result as SubprocessResult<M>;
  }

  private handleMessage(msg: unknown): void {
    if (isRpcResponse(msg)) {
      const pending = this.pending.get(msg.id);
      if (!pending) return;

      clearTimeout(pending.timeout);
      this.pending.delete(msg.id);

      if (msg.ok) {
        pending.resolve(msg.result);
      } else {
        const e = new Error(msg.error.message);
        e.name = msg.error.name;
        (e as { stack?: string }).stack = msg.error.stack;
        (e as { code?: string }).code = msg.error.code;
        pending.reject(e);
      }
      return;
    }

    if (isSubprocessEvent(msg)) {
      this.handleEvent(msg);
    }
  }

  private handleEvent(evt: SubprocessEvent): void {
    switch (evt.event) {
      case 'render:Event':
        for (const h of this.renderHandlers) h(evt.payload);
        return;
      case 'process:Ready':
        return;
      default: {
        /* empty */
      }
    }
  }

  // Returns the absolute path to the packaged Node runtime that the subprocess should use.
  // - In packaged apps: binary is shipped via electron-builder extraResources and available under process.resourcesPath.
  // - In development: the downloader stores the binary in the project root (same directory as package.json).
  private getSubprocessNodeExecPath(): string {
    if (app.isPackaged) {
      return path.join(process.resourcesPath, 'subprocess-node.exe');
    }
    // In dev, app.getAppPath() points to the project directory (where package.json lives)
    return path.join(app.getAppPath(), 'subprocess-node.exe');
  }

  // Returns the absolute path to the packaged subprocess source file.
  // - In packaged apps: the compiled file is shipped via electron-builder extraResources and available under process.resourcesPath.
  // - In development: the file is located in the build directory root
  private getSubprocessNodeSourcePath(): string {
    if (app.isPackaged) {
      return path.join(process.resourcesPath, 'build', 'tracer-subprocess.mjs');
    }
    // In dev, app.getAppPath() points to the project directory (where package.json lives)
    return path.join(app.getAppPath(), 'build', 'tracer-subprocess.mjs');
  }
}
