import { ModelChannels } from './model-channels';
import { TracerChannels } from './tracer-channels';
import {
  ElectronIpcMainProcessError,
  type ElectronIpcMainProcessErrorDto,
} from './electron-main-process-error';

const allChannelDefinitions = {
  ...ModelChannels,
  ...TracerChannels,
};
const allowedChannelNames = new Set(Object.keys(allChannelDefinitions));

type ChannelDefinitionsType = typeof allChannelDefinitions;

type ChannelNames = keyof ChannelDefinitionsType;
type Args<CN extends ChannelNames> = ChannelDefinitionsType[CN]['args'];
type DataType<CN extends ChannelNames> = ChannelDefinitionsType[CN]['dataType'];

type Result<CN extends ChannelNames> =
  | {
      data: DataType<CN>;
      error?: never;
    }
  | {
      data?: never;
      error: Error;
    };

type SerializableResult<CN extends ChannelNames> =
  | {
      data: DataType<CN>;
      error?: never;
    }
  | {
      data?: never;
      error: ElectronIpcMainProcessErrorDto;
    };

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

export { ModelChannels, allowedChannelNames, ElectronIpcMainProcessError };
export type {
  Result,
  SerializableResult,
  Args,
  DataType,
  ChannelNames,
  ChannelDefinitionsType,
  RenderStatus,
};
