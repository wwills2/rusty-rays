import { ipcMain, type IpcMainInvokeEvent } from 'electron';
import { initModelChannels } from './model';
import type {
  Args,
  ChannelNames,
  Result,
  SerializableResult,
} from '#/electron-ipc/shared';
import { initTracerChannels } from '#/electron-ipc/handles/tracer';
import { ElectronIpcMainProcessError } from '#/electron-ipc/shared/electron-main-process-error';

type ChannelListener<CN extends ChannelNames> = (
  event: IpcMainInvokeEvent,
  args: Args<CN>,
) => ChannelListenerResult<CN> | Promise<ChannelListenerResult<CN>>;

type ChannelListenerResult<CN extends ChannelNames> = Omit<Result<CN>, 'error'>;

function handle<CN extends ChannelNames>(
  channel: CN,
  listener: ChannelListener<CN>,
) {
  const tryListener = async (
    event: IpcMainInvokeEvent,
    args: Args<CN>,
  ): Promise<SerializableResult<CN>> => {
    try {
      return (await listener(event, args)) as SerializableResult<CN>;
    } catch (error) {
      if (error instanceof ElectronIpcMainProcessError) {
        return error.asResultSerialized;
      } else {
        return ElectronIpcMainProcessError.fromCaught(error, channel)
          .asResultSerialized;
      }
    }
  };

  ipcMain.handle(channel, tryListener);
}

function initIpcChannels() {
  initModelChannels();
  initTracerChannels();
}

export { handle, initIpcChannels };
