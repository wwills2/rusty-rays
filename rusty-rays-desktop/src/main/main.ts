import {app, BrowserWindow} from 'electron'
import 'dotenv/config'

function createWindow() {
  const win = new BrowserWindow({
    width: 1200,
    height: 675,
    webPreferences: {
      contextIsolation: false,
      nodeIntegration: true,
      webviewTag: true,
    },
  });

  if (process.env.ELECTRON_MAIN_ENV === 'development') {
    console.log('loading app from dev node server');
    win.loadURL('http://localhost:5173/'); // Development URL
  } else {
    throw new Error('production not set up yet')
  }
}

app.whenReady().then(createWindow)