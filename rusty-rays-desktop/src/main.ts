import { app, BrowserWindow, session, shell } from 'electron';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { initIpcChannels } from '#/ipc/handles';

// ESM -> manually define __filename and __dirname
export const __filename = fileURLToPath(import.meta.url);
export const __dirname = path.dirname(__filename);

function setConentSecurityPolicy(policy: string[]) {
  session.defaultSession.webRequest.onHeadersReceived((details, callback) => {
    callback({
      responseHeaders: {
        ...details.responseHeaders,
        'Content-Security-Policy': policy,
      },
    });
  });
}

function createMainWindow() {
  const window = new BrowserWindow({
    width: 1200,
    height: 675,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      nodeIntegration: true,
      webviewTag: true,
    },
  });

  window.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url).catch((error: unknown) => {
      console.error('failed to open link externally:', error);
    });
    return { action: 'deny' }; // Prevent the app from opening the URL internally
  });

  const setupReload = (filePath: string) => {
    // app is a SPA. reload index.html when the app tries to load a virtual route
    window.webContents.on('did-fail-load', () => {
      window.loadFile(filePath).catch((err: unknown) => {
        console.error('Failed to load index.html:', err);
        process.exit(1);
      });
    });
  };

  if (process.env.SPA_SRC === 'vite') {
    console.log('loading app from dev node server');
    window.loadURL('http://localhost:5173/').catch((err: unknown) => {
      throw err;
    }); // Development URL
  } else {
    const indexHtmlPath = path.join(__dirname, 'renderer', 'index.html');
    window
      .loadFile(indexHtmlPath)
      .then(() => {
        setupReload(indexHtmlPath);
      })
      .catch((err: unknown) => {
        throw err;
      });
  }
}

app.commandLine.appendSwitch('enable-features', 'OverlayScrollbar');

app
  .whenReady()
  .then(() => {
    if (process.env.SPA_SRC !== 'vite') {
      setConentSecurityPolicy(["default-src 'none'"]);
    }

    initIpcChannels();
    createMainWindow();

    app.on('activate', () => {
      if (BrowserWindow.getAllWindows().length === 0) {
        createMainWindow();
      }
    });
  })
  .catch((err: unknown) => {
    console.error('failed to create main window', err);
    process.exit(1);
  });

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
