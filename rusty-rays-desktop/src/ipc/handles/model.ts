import { handle, toIpcError } from './index';
import { Model } from 'rusty-rays-napi-node';
import { setModel, getModel } from '#/model-manager';

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
