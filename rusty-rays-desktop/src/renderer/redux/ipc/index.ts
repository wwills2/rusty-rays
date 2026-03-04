import { createApi } from '@reduxjs/toolkit/query/react';
import { fakeBaseQuery } from '@reduxjs/toolkit/query';
/**
 * Importing a renderer <-> main process shared resource. Need relative path
 */
import {
  allowedChannelNames,
  type Args,
  type ChannelNames,
  type DataType,
  type Result,
  toIpcError,
} from '../../../electron-ipc/shared';

function processIpcResult<CN extends ChannelNames, R>(
  channelName: CN,
  result: Result<CN>,
  operation: (data: DataType<CN>) => R,
) {
  if (result.data !== undefined && result.error === undefined) {
    return { data: operation(result.data) };
  } else {
    if (result.error instanceof Error) {
      throw new Error(
        `IPC channel ${channelName} returned error: ${result.error.message}`,
        {
          cause: result.error,
        },
      );
    } else {
      throw new Error(
        `IPC channel ${channelName} returned unknown error: ${JSON.stringify(result.error)}`,
      );
    }
  }
}

async function invoke<CN extends ChannelNames>(
  channel: CN,
  ...args: Args<CN>
): Promise<Result<CN>> {
  if (!allowedChannelNames.has(channel)) {
    throw new Error(`channel ${channel} is not in set of allowed channels`);
  }

  try {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-expect-error
    // eslint-disable-next-line @typescript-eslint/no-unsafe-call,@typescript-eslint/no-unsafe-member-access
    const result = (await window.electronAPI.allowedChannelInvoke(
      channel,
      args,
    )) as unknown as Result<CN>;

    console.debug('electron-ipc channel and returned data', channel, result);
    return result;
  } catch (error) {
    return toIpcError(
      error,
      'an error occurred during electron-ipc invocation',
    );
  }
}

const ipcApi = createApi({
  reducerPath: 'ipcApi',
  endpoints: () => ({}),
  baseQuery: fakeBaseQuery(),
  tagTypes: [...allowedChannelNames],
});

export { ipcApi, invoke, processIpcResult };
export type { DataType };
