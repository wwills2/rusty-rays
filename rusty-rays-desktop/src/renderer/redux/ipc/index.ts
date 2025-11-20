import { createApi } from '@reduxjs/toolkit/query/react';
import { fakeBaseQuery } from '@reduxjs/toolkit/query';
import {
  allowedChannelNames,
  toIpcError,
  type Args,
  type ChannelNames,
  type Result,
} from '../../../ipc/shared';

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

    console.log('*****', channel, result);
    return result;
  } catch (error) {
    return toIpcError(error, 'an error occurred during ipc invocation');
  }
}

const ipcApi = createApi({
  reducerPath: 'ipcApi',
  endpoints: () => ({}),
  baseQuery: fakeBaseQuery(),
});

export { ipcApi, invoke };
