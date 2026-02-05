// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts
import { ipcRenderer, contextBridge } from 'electron/renderer';
import { allowedChannelNames } from '#/ipc/shared';

contextBridge.exposeInMainWorld('electronAPI', {
  allowedChannelInvoke: async (channel: string, args: unknown) => {
    if (!allowedChannelNames.has(channel)) {
      throw new Error(`channel ${channel} is not in set of allowed channels`);
    }

    // this in the preload context, so typing doesn't matter anyway
    // allow any here
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    return await ipcRenderer.invoke(channel, args);
  },
});

contextBridge.exposeInMainWorld('env', {
  isVite: process.env.SPA_SRC === 'vite',
});
