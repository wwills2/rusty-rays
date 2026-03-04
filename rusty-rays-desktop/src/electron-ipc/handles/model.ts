// src/electron-ipc/handles/model.ts
import { handle } from './index';
import {
  getAllCones,
  getAllPolygons,
  getAllSpheres,
  getAllTriangles,
  getTracerInstance,
  setModel,
  setModelFromFilePath,
  setModelFromFileTextString
} from '#/tracer-manager';
import { toIpcError } from '#/electron-ipc/shared';

function initModelChannels() {
  handle('model:InitFromFilePath', async (_, args) => {
    try {
      const [path] = args;
      const instanceUuid = await setModelFromFilePath(path);
      return { data: { instanceUuid } };
    } catch (error) {
      return toIpcError(error, 'failed to initialize model from file');
    }
  });

  handle('model:InitFromFileTextString', async (_, args) => {
    try {
      const [fileText] = args;
      const instanceUuid = await setModelFromFileTextString(fileText);
      return { data: { instanceUuid } };
    } catch (error) {
      return toIpcError(error, 'failed to initialize model from file text');
    }
  });

  // Keep your existing IPC channel names, but fetch from subprocess.
  handle('model:getAllSpheres', async () => {
    try {
      const instance = getTracerInstance();
      if (!instance) {
        return toIpcError(
          new Error('failed to fetch spheres. no model loaded'),
          '',
        );
      }
      const data = await getAllSpheres();
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
      const data = await getAllCones();
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
      const data = await getAllTriangles();
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
      const data = await getAllPolygons();
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
      }

      // still unimplemented
      return toIpcError(new Error(`cannot load model with uuid ${uuid}`), '');
    } catch (error) {
      return toIpcError(error, 'failed to set model');
    }
  });
}

export { initModelChannels };
