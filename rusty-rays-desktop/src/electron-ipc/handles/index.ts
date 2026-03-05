import { ipcMain, type IpcMainInvokeEvent } from 'electron';
import { initModelChannels } from './model';
import type { Args, ChannelNames, Result } from '#/electron-ipc/shared';
import { initTracerChannels } from '#/electron-ipc/handles/tracer';

type ChannelListener<CN extends ChannelNames> = (
  event: IpcMainInvokeEvent,
  args: Args<CN>,
) => Result<CN> | Promise<Result<CN>>;

function handle<CN extends ChannelNames>(
  channel: CN,
  listener: ChannelListener<CN>,
) {
  ipcMain.handle(channel, listener);
}

function initIpcChannels() {
  initModelChannels();
  initTracerChannels();
}

export { handle, initIpcChannels };
