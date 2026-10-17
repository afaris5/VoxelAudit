import process from 'node:process';
import { spawnSync } from 'node:child_process';
import { homedir } from 'node:os';
import { resolve } from 'node:path';
import { fileURLToPath, URL } from 'node:url';
import { nativeEnvironment } from './native-env.mjs';

const existing = process.env.CARGO_ENCODED_RUSTFLAGS;
if (!existing && process.env.RUSTFLAGS) {
  throw new Error(
    'Use CARGO_ENCODED_RUSTFLAGS for custom build flags so path remapping can be preserved safely.',
  );
}
const flags = [
  ...(existing ? existing.split('\x1f') : []),
  `--remap-path-prefix=${homedir()}=build-home`,
  `--remap-path-prefix=${resolve('.')}=workspace`,
];
const result = spawnSync(
  process.execPath,
  [
    fileURLToPath(
      new URL('../node_modules/@tauri-apps/cli/tauri.js', import.meta.url),
    ),
    'build',
    ...process.argv.slice(2),
  ],
  {
    stdio: 'inherit',
    env: {
      ...nativeEnvironment(),
      CARGO_ENCODED_RUSTFLAGS: flags.join('\x1f'),
    },
  },
);
if (result.error) throw result.error;
process.exit(result.status ?? 1);
