import { ModelChannels } from './model-channels';
import { TracerChannels } from './tracer-channels';

const allChannelDefinitions = {
  ...ModelChannels,
  ...TracerChannels,
};
const allowedChannelNames = new Set(Object.keys(allChannelDefinitions));

type ChannelDefinitionsType = typeof allChannelDefinitions;

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

/**
 * Type for the status of the render process managed by the tracer manager
 */
type RenderStatus = {
  tracerInstanceUuid?: string;
  renderProgressPercentage?: number;
  writingImage: boolean;
  renderErrorMsg?: string;
  renderImageAvailable: boolean;
};

export { ModelChannels, toIpcError, allowedChannelNames };
export type {
  Result,
  Args,
  DataType,
  ChannelNames,
  ChannelDefinitionsType,
  RenderStatus,
};
