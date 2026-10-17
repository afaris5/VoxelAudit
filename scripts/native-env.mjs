import process from 'node:process';
import { existsSync } from 'node:fs';
import { delimiter, join, resolve } from 'node:path';

export function nativeEnvironment() {
  const env = { ...process.env };
  const local = resolve('.toolchain');
  const cargo = join(local, 'cargo', 'bin');
  if (
    !env.CARGO_HOME &&
    !env.RUSTUP_HOME &&
    existsSync(
      join(cargo, process.platform === 'win32' ? 'cargo.exe' : 'cargo'),
    )
  ) {
    env.CARGO_HOME = join(local, 'cargo');
    env.RUSTUP_HOME = join(local, 'rustup');
    const pathKey =
      Object.keys(env).find((key) => key.toLowerCase() === 'path') ?? 'PATH';
    env[pathKey] = `${cargo}${delimiter}${env[pathKey] ?? ''}`;
  }
  return env;
}
