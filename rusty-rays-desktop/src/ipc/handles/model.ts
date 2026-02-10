import { handle } from './index';
import { getTracerInstance, setModel } from '#/tracer-manager';
import { Model } from 'rusty-rays-napi-node';
import { toIpcError } from '#/ipc/shared';

function initModelChannels() {
  handle('model:InitFromFilePath', async (_, args) => {
    try {
      const [path] = args;
      const model = await Model.fromFilePath(path);
      const instanceUuid = setModel(model);
      if (!instanceUuid) {
        return toIpcError(
          new Error(`invalid tracer instance uuid. recieved ${instanceUuid}`),
          '',
        );
      }

      return { data: { instanceUuid } };
    } catch (error) {
      return toIpcError(error, 'failed to initialize model from file');
    }
  });

  handle('model:InitFromFileTextString', (_, args) => {
    try {
      const [fileText] = args;
      const model = Model.fromString(fileText);
      const instanceUuid = setModel(model);
      if (!instanceUuid) {
        return toIpcError(
          new Error(`invalid tracer instance uuid. received ${instanceUuid}`),
          '',
        );
      }

      return { data: { instanceUuid } };
    } catch (error) {
      return toIpcError(error, 'failed to initialize model from file text');
    }
  });

  handle('model:getAllSpheres', async () => {
    try {
      const instance = getTracerInstance();
      if (!instance) {
        return toIpcError(
          new Error('failed to fetch spheres. no model loaded'),
          '',
        );
      }

      const { model } = instance;
      const data = await model.allSpheres;

      return { data };
    } catch (error) {
      return toIpcError(error, 'failed to fetch spheres');
    }
  });
}

export { initModelChannels };
