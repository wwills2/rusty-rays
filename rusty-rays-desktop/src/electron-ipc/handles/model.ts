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
  setModelFromFileTextString,
} from '#/tracer-manager';

function initModelChannels() {
  handle('model:InitFromFilePath', async (_, args) => {
    const [path] = args;
    const instanceUuid = await setModelFromFilePath(path);
    return { data: { instanceUuid } };
  });

  handle('model:InitFromFileTextString', async (_, args) => {
    const [fileText] = args;
    const instanceUuid = await setModelFromFileTextString(fileText);
    return { data: { instanceUuid } };
  });

  // Keep your existing IPC channel names, but fetch from subprocess.
  handle('model:getAllSpheres', async () => {
    const instance = getTracerInstance();
    if (!instance) {
      throw new Error('failed to fetch spheres. no model loaded');
    }
    const data = await getAllSpheres();
    return { data };
  });

  handle('model:getAllCones', async () => {
    const instance = getTracerInstance();
    if (!instance) {
      throw new Error('failed to fetch cones. no model loaded');
    }
    const data = await getAllCones();
    return { data };
  });

  handle('model:getAllTriangles', async () => {
    const instance = getTracerInstance();
    if (!instance) {
      throw new Error('failed to fetch triangles. no model loaded');
    }
    const data = await getAllTriangles();
    return { data };
  });

  handle('model:getAllPolygons', async () => {
    const instance = getTracerInstance();
    if (!instance) {
      throw new Error('failed to fetch spheres. no model loaded');
    }
    const data = await getAllPolygons();
    return { data };
  });

  handle('model:SetModel', async (_, args) => {
    const [uuid] = args;
    if (uuid === undefined) {
      await setModel(undefined);
      return { data: true };
    }

    // still unimplemented
    throw new Error(`cannot load model with uuid ${uuid}`);
  });
}

export { initModelChannels };
