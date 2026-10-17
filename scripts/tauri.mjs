import process from 'node:process';
import { spawnSync } from 'node:child_process';
import { fileURLToPath, URL } from 'node:url';
import { nativeEnvironment } from './native-env.mjs';

const result = spawnSync(
  process.execPath,
  [
    fileURLToPath(
      new URL('../node_modules/@tauri-apps/cli/tauri.js', import.meta.url),
    ),
    ...process.argv.slice(2),
  ],
  { stdio: 'inherit', env: nativeEnvironment() },
);
if (result.error) throw result.error;
process.exit(result.status ?? 1);
