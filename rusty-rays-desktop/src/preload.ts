// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts
import { ipcRenderer, contextBridge } from 'electron/renderer';
import { allowedChannelNames } from '#/ipc/shared';

contextBridge.exposeInMainWorld('electronAPI', {
  allowedChannelInvoke: async (channel: string, args: unknown) => {
    if (!allowedChannelNames.has(channel)) {
      throw new Error(`channel ${channel} is not in set of allowed channels`);
    }

    try {
      return await ipcRenderer.invoke(channel, args);
    } catch (error) {
      throw error;
    }
  },
});
