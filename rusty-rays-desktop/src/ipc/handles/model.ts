import { handle } from './index';
import { setModel, getModel } from '#/model-manager';
import { Model } from 'rusty-rays-napi-node';
import { toIpcError } from '#/ipc/shared';

function initModelChannels() {
  handle('model:InitFromFile', async (_, args) => {
    try {
      const [path] = args;
      const model = await Model.fromFilePath(path);
      setModel(model);
      return { data: true };
    } catch (error) {
      return toIpcError(error, 'failed to initialize model from file');
    }
  });

  handle('model:getAllSpheres', async () => {
    try {
      const data = await getModel()?.allSpheres;
      return { data };
    } catch (error) {
      return toIpcError(error, 'failed to fetch spheres');
    }
  });
}

export { initModelChannels };
