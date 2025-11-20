import type { ModelChannels } from './model-channels';
import IpcMainInvokeEvent = Electron.Main.IpcMainInvokeEvent;

export type ChannelDefinitions = ModelChannels;

export type ChannelNames = keyof ChannelDefinitions;

export type ChannelListener<CN extends ChannelNames> = (
  event: IpcMainInvokeEvent,
  ...args: ChannelDefinitions[CN]['args']
) =>
  | Result<ChannelDefinitions[CN]['result']>
  | Promise<Result<ChannelDefinitions[CN]['result']>>;

export type Result<D> = {
  data?: D;
  error?: Error;
};
