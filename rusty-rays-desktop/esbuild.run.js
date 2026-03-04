import esbuild from 'esbuild';
import { mkdir } from 'node:fs/promises';
import { builtinModules } from 'node:module';
import path, { resolve } from 'path';
import { fileURLToPath } from 'url';
import copy from 'esbuild-plugin-copy';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function main() {
  try {
    await mkdir('build');
  } catch {}

  const sharedConfig = {
    alias: {
      '#': resolve(__dirname, 'src'),
    },
    platform: 'node',
    format: 'esm',
    target: 'es2023',
    bundle: true,
    sourcemap: true,
    minify: false,
    external: [
      'electron',
      ...builtinModules.map((m) => `node:${m}`),
      ...builtinModules,
    ],
  };

  // Build main process (Electron)
  await esbuild.build({
    ...sharedConfig,
    entryPoints: ['src/main.ts'],
    outfile: 'build/index.js',
    plugins: [
      copy({
        globbyOptions: {
          expandDirectories: true,
          gitignore: false,
        },
        assets: {
          from: ['./node_modules/rusty-rays-napi-node/bindings/**/*'],
          to: [resolve(__dirname, 'build/bindings')],
        },
        verbose: true,
      }),
    ],
  });

  // Build preload script
  await esbuild.build({
    ...sharedConfig,
    entryPoints: ['src/preload.ts'],
    outfile: 'build/preload.js',
  });

  // Build tracer subprocess (Node-only worker)
  await esbuild.build({
    ...sharedConfig,
    entryPoints: ['src/tracer-subprocess.ts'],
    outfile: 'build/tracer-subprocess.mjs',
    // optional but often helpful if you want to ensure it stays "node-ish"
    // conditions: ['node'],
  });

  console.log('Built main -> build/index.js');
  console.log('Built preload -> build/preload.js');
  console.log('Built tracer subprocess -> build/tracer-subprocess.mjs');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
