import { handle } from './index';
import { getTracerInstance, setModel } from '#/tracer-manager';
import { Model } from 'rusty-rays-napi-node';
import { toIpcError } from '#/ipc/shared';

function initModelChannels() {
  handle('model:InitFromFilePath', async (_, args) => {
    try {
      const [path] = args;
      const model = await Model.fromFilePath(path);
      const instanceUuid = await setModel(model);
      if (!instanceUuid) {
        return toIpcError(
          new Error(`invalid tracer instance uuid. received ${instanceUuid}`),
          '',
        );
      }

      return { data: { instanceUuid } };
    } catch (error) {
      return toIpcError(error, 'failed to initialize model from file');
    }
  });

  handle('model:InitFromFileTextString', async (_, args) => {
    try {
      const [fileText] = args;
      const model = Model.fromString(fileText);
      const instanceUuid = await setModel(model);
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

  handle('model:getAllCones', async () => {
    try {
      const instance = getTracerInstance();
      if (!instance) {
        return toIpcError(
          new Error('failed to fetch cones. no model loaded'),
          '',
        );
      }

      const { model } = instance;
      const data = await model.allCones;

      return { data };
    } catch (error) {
      return toIpcError(error, 'failed to fetch cones');
    }
  });

  handle('model:getAllTriangles', async () => {
    try {
      const instance = getTracerInstance();
      if (!instance) {
        return toIpcError(
          new Error('failed to fetch triangles. no model loaded'),
          '',
        );
      }

      const { model } = instance;
      const data = await model.allTriangles;

      return { data };
    } catch (error) {
      return toIpcError(error, 'failed to fetch triangles');
    }
  });

  handle('model:getAllPolygons', async () => {
    try {
      const instance = getTracerInstance();
      if (!instance) {
        return toIpcError(
          new Error('failed to fetch polygons. no model loaded'),
          '',
        );
      }

      const { model } = instance;
      const data = await model.allPolygons;

      return { data };
    } catch (error) {
      return toIpcError(error, 'failed to fetch polygons');
    }
  });

  handle('model:SetModel', async (_, args) => {
    try {
      const [uuid] = args;
      if (uuid === undefined) {
        await setModel(undefined);
        return { data: true };
      } else {
        // todo load the model corresponding to the uuid if provided. functionality yet to be implemented
        return toIpcError(new Error(`cannot load model with uuid ${uuid}`), '');
      }
    } catch (error) {
      return toIpcError(error, 'failed to set model');
    }
  });
}

export { initModelChannels };
