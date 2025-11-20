import { ipcMain } from 'electron';
import { initModelChannels } from './model';
import type { ChannelNames, ChannelListener, Result } from '../shared-types';

function handle<CN extends ChannelNames>(
  channel: CN,
  listener: ChannelListener<CN>,
) {
  ipcMain.handle(channel, listener);
}

function toIpcError<D>(error: unknown, altMsg: string): Result<D> {
  if (error instanceof Error) {
    return { error };
  } else {
    return { error: new Error(altMsg) };
  }
}

function initIpcChannels() {
  initModelChannels();
}

export { handle, initIpcChannels, toIpcError };
