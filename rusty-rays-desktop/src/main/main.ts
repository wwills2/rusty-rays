import "dotenv/config";
import { app, BrowserWindow } from "electron";
import * as path from "path";
import { fileURLToPath } from "url";

// ESM -> manually define __filename and __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function createWindow() {
  const window = new BrowserWindow({
    width: 1200,
    height: 675,
    webPreferences: {
      nodeIntegration: true,
      webviewTag: true,
    },
  });

  const setupReload = (filePath: string) => {
    // app is a SPA. reload index.html when the app tries to load a virtual route
    window.webContents.on("did-fail-load", () => {
      window.loadFile(filePath).catch((err: unknown) => {
        console.error("Failed to load index.html:", err);
        process.exit(1);
      });
    });
  };

  if (process.env.SPA_SRC === "vite") {
    console.log("loading app from dev node server");
    window.loadURL("http://localhost:5173/").catch((err: unknown) => {
      throw err;
    }); // Development URL
  } else {
    const indexHtmlPath = path.join(__dirname, "renderer", "index.html");
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

void app.whenReady().then(createWindow);
