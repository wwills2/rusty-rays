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
  ElectronIpcMainProcessError,
  type Result,
  type SerializableResult,
} from '../../../electron-ipc/shared';

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
    )) as unknown as SerializableResult<CN>;

    console.debug(`electron-ipc channel ${channel} returned result:`, result);

    if (result.error) {
      return ElectronIpcMainProcessError.fromObject(result.error).asResult;
    } else if (!result.data) {
      return {
        error: new ElectronIpcMainProcessError(
          'Invalid result from electron-ipc invocation',
          channel,
        ),
      };
    }

    return result;
  } catch (error) {
    const _error = ElectronIpcMainProcessError.fromCaught(
      error,
      channel,
      'an error occurred during electron-ipc invocation',
    );

    return { error: _error };
  }
}

const ipcApi = createApi({
  reducerPath: 'ipcApi',
  endpoints: () => ({}),
  baseQuery: fakeBaseQuery<Error>(),
  tagTypes: [...allowedChannelNames],
});

export { ipcApi, invoke };
export type { DataType };
