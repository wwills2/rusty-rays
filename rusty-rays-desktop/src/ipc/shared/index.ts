import { ModelChannels } from './model-channels';

const allChannelDefinitions = {
  ...ModelChannels,
};
const allowedChannelNames = new Set(Object.keys(allChannelDefinitions));

type ChannelDefinitionsType = typeof ModelChannels;
type ChannelNames = keyof ChannelDefinitionsType;
type Args<CN extends ChannelNames> = ChannelDefinitionsType[CN]['args'];
type DataType<CN extends ChannelNames> = ChannelDefinitionsType[CN]['dataType'];

type Result<CN extends ChannelNames> = {
  data?: DataType<CN>;
  error?: Error;
};

function toIpcError(error: unknown, altMsg: string) {
  if (error instanceof Error) {
    return { error };
  } else {
    return { error: new Error(altMsg) };
  }
}

export { ModelChannels, toIpcError, allowedChannelNames };
export type { Result, Args, DataType, ChannelNames };
