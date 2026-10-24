import { type ChannelNames, type Result } from './';

interface ErrorDto {
  readonly name: string;
  readonly message: string;
  readonly stack?: string;
  readonly cause?: ErrorDto;
}

interface ElectronIpcMainProcessErrorDto {
  readonly kind: 'electron-ipc-main-error';
  readonly version: 1;
  readonly channelName: ChannelNames;
  readonly serializedError: ErrorDto;
}

function describeThrownValue(value: unknown): string {
  const preamble = 'Caught non-error value:';

  try {
    return `${preamble} ${String(value)}`;
  } catch {
    return `${preamble} [Unprintable thrown value]`;
  }
}

function serializeError(
  value: unknown,
  seen = new WeakSet<object>(),
  depth = 0,
): ErrorDto {
  if (!(value instanceof Error)) {
    return {
      name: 'NonErrorThrown',
      message: describeThrownValue(value),
    };
  }

  if (seen.has(value)) {
    return { name: 'Error', message: '[Circular error cause]' };
  }

  if (depth >= 20) {
    return { name: 'Error', message: '[Error cause depth exceeded]' };
  }

  seen.add(value);

  try {
    const stack = value.stack;

    return {
      name: value.name,
      message: value.message,
      ...(typeof stack === 'string' ? { stack } : {}),
      ...(Object.hasOwn(value, 'cause')
        ? { cause: serializeError(value.cause, seen, depth + 1) }
        : {}),
    };
  } finally {
    seen.delete(value);
  }
}

function restoreStack(error: Error, stack: string | undefined): void {
  if (stack === undefined) {
    delete error.stack;
  } else {
    error.stack = stack;
  }
}

function deserializeError(dto: ErrorDto): Error {
  const error = new Error(
    dto.message,
    dto.cause === undefined
      ? undefined
      : { cause: deserializeError(dto.cause) },
  );

  Object.defineProperty(error, 'name', {
    value: dto.name,
    configurable: true,
    writable: true,
  });

  restoreStack(error, dto.stack);

  return error;
}

class ElectronIpcMainProcessError extends Error {
  readonly channelName: ChannelNames;

  constructor(
    message: string,
    channelName: ChannelNames,
    options?: ErrorOptions,
  ) {
    super(message, options);

    this.channelName = channelName;

    Object.defineProperty(this, 'name', {
      value: 'ElectronIpcMainProcessError',
      configurable: true,
      writable: true,
    });
  }

  static fromCaught(
    value: unknown,
    channelName: ChannelNames,
    message?: string,
  ): ElectronIpcMainProcessError {
    const constructMessage = () => {
      if (value instanceof Error) {
        return value.message;
      }
      return describeThrownValue(value);
    };

    const _message = message ?? constructMessage();

    return new ElectronIpcMainProcessError(_message, channelName, {
      cause: value,
    });
  }

  get asResult(): Result<ChannelNames> {
    return { error: this };
  }

  get asResultSerialized() {
    return { error: this.toObject() };
  }

  toObject(): ElectronIpcMainProcessErrorDto {
    return {
      kind: 'electron-ipc-main-error',
      version: 1,
      channelName: this.channelName,
      serializedError: serializeError(this),
    };
  }

  static fromObject(
    dto: ElectronIpcMainProcessErrorDto,
  ): ElectronIpcMainProcessError {
    const data = dto.serializedError;

    const error = new ElectronIpcMainProcessError(
      data.message,
      dto.channelName,
      data.cause === undefined
        ? undefined
        : { cause: deserializeError(data.cause) },
    );

    restoreStack(error, data.stack);

    return error;
  }
}

export { ElectronIpcMainProcessError };
export type { ElectronIpcMainProcessErrorDto };
