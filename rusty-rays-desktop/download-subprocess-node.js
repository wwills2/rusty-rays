/*
  download-subprocess-node.js
  - Reads .nvmrc to determine Node version (major[.minor[.patch]])
  - Resolves to the latest patch for that major/minor using Node index.json
  - Downloads the official Node.js distribution for the current OS/arch
  - Extracts it and copies the node binary to ./subprocess-node.exe
  - Skips work if the correct version is already present
*/

import fs from 'fs';
import path from 'path';
import os from 'os';
import https from 'https';
import { spawnSync } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const projectDir = path.dirname(__filename);
const nvmrcPath = path.join(projectDir, '.nvmrc');
const outBinaryName = 'subprocess-node.exe'; // required name on all platforms
const outBinaryPath = path.join(projectDir, outBinaryName);
const versionMarkerPath = path.join(projectDir, 'subprocess-node.version');

function readNvmrc() {
  if (!fs.existsSync(nvmrcPath)) {
    throw new Error('.nvmrc not found. Please add a Node version to .nvmrc');
  }
  const raw = fs.readFileSync(nvmrcPath, 'utf8').trim();
  if (!raw) throw new Error('.nvmrc is empty');
  return raw.replace(/^v/, '');
}

function fetchJson(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (res) => {
        if (
          res.statusCode &&
          res.statusCode >= 300 &&
          res.statusCode < 400 &&
          res.headers.location
        ) {
          // follow redirect
          fetchJson(res.headers.location).then(resolve, reject);
          return;
        }
        if (res.statusCode !== 200) {
          reject(
            new Error(`Failed to fetch ${url} (status ${res.statusCode})`),
          );
          return;
        }
        const chunks = [];
        res.on('data', (d) => chunks.push(d));
        res.on('end', () => {
          try {
            const text = Buffer.concat(chunks).toString('utf8');
            resolve(JSON.parse(text));
          } catch (e) {
            reject(e);
          }
        });
      })
      .on('error', reject);
  });
}

function pickVersion(base) {
  // base could be: "24", "24.11", or full "24.11.0"
  const segments = base
    .split('.')
    .map((s) => s.trim())
    .filter(Boolean);
  if (segments.length === 3) return base; // already full

  return fetchJson('https://nodejs.org/dist/index.json').then((index) => {
    // index is array of release objects with version like 'v24.11.0'
    const targetMajor = parseInt(segments[0], 10);
    const targetMinor =
      segments.length > 1 ? parseInt(segments[1], 10) : undefined;

    const candidates = index
      .map((r) => r.version.replace(/^v/, ''))
      .filter((v) => {
        const [maj, min] = v.split('.').map((n) => parseInt(n, 10));
        if (maj !== targetMajor) return false;
        if (typeof targetMinor === 'number' && min !== targetMinor)
          return false;
        return true;
      })
      .sort((a, b) => {
        const pa = a.split('.').map((n) => parseInt(n, 10));
        const pb = b.split('.').map((n) => parseInt(n, 10));
        // descending
        for (let i = 0; i < 3; i++) {
          if (pb[i] !== pa[i]) return pb[i] - pa[i];
        }
        return 0;
      });

    if (!candidates.length) {
      throw new Error(`No Node versions found matching ${base}`);
    }
    return candidates[0];
  });
}

function mapPlatformArch() {
  const platform = process.platform; // 'win32', 'darwin', 'linux'
  const arch = process.arch; // 'x64', 'arm64', 'arm', etc.

  let nodePlatform;
  let nodeArch;
  let ext;

  if (platform === 'win32') {
    nodePlatform = 'win';
    ext = 'zip';
    if (arch === 'x64') nodeArch = 'x64';
    else if (arch === 'arm64') nodeArch = 'arm64';
    else if (arch === 'ia32') nodeArch = 'x86';
  } else if (platform === 'darwin') {
    nodePlatform = 'darwin';
    ext = 'tar.xz';
    if (arch === 'x64') nodeArch = 'x64';
    else if (arch === 'arm64') nodeArch = 'arm64';
  } else if (platform === 'linux') {
    nodePlatform = 'linux';
    ext = 'tar.xz';
    if (arch === 'x64') nodeArch = 'x64';
    else if (arch === 'arm64') nodeArch = 'arm64';
    else if (arch === 'arm') nodeArch = 'armv7l';
  }

  if (!nodePlatform || !nodeArch) {
    throw new Error(`Unsupported platform/arch: ${platform}/${arch}`);
  }

  return { nodePlatform, nodeArch, ext };
}

function downloadFile(url, destPath) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(destPath);
    https
      .get(url, (res) => {
        if (
          res.statusCode &&
          res.statusCode >= 300 &&
          res.statusCode < 400 &&
          res.headers.location
        ) {
          // follow redirect
          res.destroy();
          downloadFile(res.headers.location, destPath).then(resolve, reject);
          return;
        }
        if (res.statusCode !== 200) {
          reject(
            new Error(`Failed to download ${url} (status ${res.statusCode})`),
          );
          return;
        }
        res.pipe(file);
        file.on('finish', () => file.close(() => resolve()));
      })
      .on('error', (err) => {
        fs.unlink(destPath, () => reject(err));
      });
  });
}

function extractArchive(archivePath, destDir, isWindows) {
  fs.mkdirSync(destDir, { recursive: true });
  if (isWindows) {
    // Use PowerShell Expand-Archive for .zip
    const cmd = 'powershell.exe';
    const args = [
      '-NoProfile',
      '-Command',
      `Expand-Archive -Path '${archivePath}' -DestinationPath '${destDir}' -Force`,
    ];
    const res = spawnSync(cmd, args, { stdio: 'inherit' });
    if (res.status !== 0)
      throw new Error('Failed to extract archive with PowerShell');
  } else {
    // Use tar for .tar.xz
    // tar -xf <archive> -C <dest>
    const res = spawnSync('tar', ['-xf', archivePath, '-C', destDir], {
      stdio: 'inherit',
    });
    if (res.status !== 0) throw new Error('Failed to extract archive with tar');
  }
}

function writeVersionMarker(value) {
  fs.writeFileSync(versionMarkerPath, value, 'utf8');
}

function readVersionMarker() {
  if (!fs.existsSync(versionMarkerPath)) return null;
  try {
    return fs.readFileSync(versionMarkerPath, 'utf8').trim();
  } catch {
    return null;
  }
}

async function main() {
  const baseVersion = readNvmrc();
  const fullVersion = await pickVersion(baseVersion); // e.g., "24.11.0"
  const { nodePlatform, nodeArch, ext } = mapPlatformArch();

  const currentMarker = readVersionMarker();
  const newMarker = `${fullVersion} ${process.platform} ${process.arch}`;
  if (fs.existsSync(outBinaryPath) && currentMarker === newMarker) {
    console.log(
      `subprocess-node already present (${newMarker}), skipping download.`,
    );
    return;
  }

  console.log(
    `Preparing Node ${fullVersion} for ${nodePlatform}-${nodeArch}...`,
  );

  const versionTag = `v${fullVersion}`;
  const baseName = `node-${versionTag}-${nodePlatform}-${nodeArch}`;
  const fileName = `${baseName}.${ext}`;
  const url = `https://nodejs.org/dist/${versionTag}/${fileName}`;

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'subprocess-node-'));
  const archivePath = path.join(tmpDir, fileName);
  await downloadFile(url, archivePath);
  console.log(`Downloaded ${url}`);

  const extractDir = path.join(tmpDir, 'extract');
  extractArchive(archivePath, extractDir, process.platform === 'win32');

  const rootDir = path.join(extractDir, baseName);
  let srcBinary;
  if (process.platform === 'win32') {
    srcBinary = path.join(rootDir, 'node.exe');
  } else {
    srcBinary = path.join(rootDir, 'bin', 'node');
  }
  if (!fs.existsSync(srcBinary)) {
    throw new Error(`Node binary not found in extracted archive: ${srcBinary}`);
  }

  // Copy/rename to desired name in project root
  fs.copyFileSync(srcBinary, outBinaryPath);
  if (process.platform !== 'win32') {
    fs.chmodSync(outBinaryPath, 0o755);
  }

  // Write marker
  writeVersionMarker(newMarker);

  console.log(`Saved Node binary to ${outBinaryPath}`);
}

main().catch((err) => {
  console.error(
    'Failed to prepare subprocess Node:',
    err && err.stack ? err.stack : err,
  );
  process.exit(1);
});
