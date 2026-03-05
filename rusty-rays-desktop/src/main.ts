import { app, BrowserWindow, nativeImage, session, shell } from 'electron';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { initIpcChannels } from '#/electron-ipc/handles';

// ESM -> manually define __filename and __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function setContentSecurityPolicy(policy: string[]) {
  session.defaultSession.webRequest.onHeadersReceived((details, callback) => {
    callback({
      responseHeaders: {
        ...details.responseHeaders,
        'Content-Security-Policy': policy,
      },
    });
  });
}

function getIconPath() {
  // In packaged apps, resources are under process.resourcesPath
  // In dev, resolve relative to project directory (build/../assets/icons)
  const base = app.isPackaged
    ? path.join(process.resourcesPath, 'assets', 'icons')
    : path.resolve(__dirname, '..', 'assets', 'icons');

  if (process.platform === 'win32') return path.join(base, 'win', 'icon.ico');
  if (process.platform === 'darwin') return path.join(base, 'mac', 'icon.icns');
  return path.join(base, 'png', '512x512.png'); // Linux
}

function createMainWindow() {
  const iconPath = getIconPath();
  const window = new BrowserWindow({
    width: 1000,
    height: 800,
    // On Windows/Linux, the window icon sets the taskbar/alt-tab icon.
    icon: process.platform === 'darwin' ? undefined : iconPath,
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

  // On macOS, BrowserWindow icon is ignored; set Dock icon instead.
  if (process.platform === 'darwin') {
    try {
      app.dock?.setIcon(nativeImage.createFromPath(iconPath));
    } catch (e) {
      console.warn('Failed to set macOS dock icon:', e);
    }
  }
}

app.commandLine.appendSwitch('enable-features', 'OverlayScrollbar');

app
  .whenReady()
  .then(() => {
    if (process.env.SPA_SRC !== 'vite') {
      setContentSecurityPolicy([
        `
  default-src 'none';
  script-src 'self';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data: file: blob:;
  font-src 'self' file:;
  connect-src 'self' https:;
  media-src 'self' file:;
  object-src 'none';
  frame-src 'none';
  base-uri 'self';
  form-action 'self';
`,
      ]);
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
