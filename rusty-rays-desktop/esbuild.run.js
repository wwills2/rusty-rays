import esbuild from 'esbuild';
import { mkdir } from 'node:fs/promises';
import { builtinModules } from 'node:module';
import { resolve } from 'path';

async function main() {
  // Ensure dist directory exists
  try {
    await mkdir('build');
  } catch {}

  // Shared config for both main and preload builds
  const sharedConfig = {
    platform: 'node',
    format: 'esm', // Change to "cjs" if you prefer CJS output
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

  // Build main process
  await esbuild.build({
    ...sharedConfig,
    entryPoints: ['src/main.ts'],
    outfile: 'build/index.js',
  });

  // Build preload script
  await esbuild.build({
    ...sharedConfig,
    entryPoints: ['src/preload.ts'],
    outfile: 'build/preload.js',
  });

  console.log('Built main → build/index.js');
  console.log('Built preload → build/preload.js');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
